//! Linux iptables interface

use crate::network::cni::port::PortMapping;
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use derive_builder::Builder;
use dyn_clone::{clone_trait_object, DynClone};
use getset::{CopyGetters, Getters, Setters};
use ipnetwork::IpNetwork;
use log::trace;
use std::{
    collections::BTreeMap,
    fmt::{self, Debug},
    net::Ipv4Addr,
    path::PathBuf,
    process::Output,
};
use tokio::process::Command;

#[async_trait]
/// Iptables behavior trait.
pub trait Iptables: DynClone + Send + Sync {
    /// Setup the provided `Chain` with respect of whether to use IPv6.
    async fn setup(&self, _chain: &Chain) -> Result<()> {
        Ok(())
    }

    /// Idempotently delete a chain. It will not error if the chain doesn't exist. It will first
    /// delete all references to this chain in the `entry_chains`.
    async fn teardown(&self, _chain: &Chain) -> Result<()> {
        Ok(())
    }

    /// Enable IPv6 usage for this iptables instance.
    fn set_ipv6(&mut self, _value: bool) {}
}

clone_trait_object!(Iptables);

#[derive(Builder, Clone, CopyGetters, Getters, Setters)]
#[builder(pattern = "owned", setter(into))]
/// The main interface to the Linux iptables.
pub struct DefaultIptables {
    #[get]
    /// Path to the `iptables` binary.
    iptables_binary: PathBuf,

    #[get]
    /// Path to the `ip6tables` binary.
    ip6tables_binary: PathBuf,

    #[getset(set, get_copy)]
    #[builder(default = "false")]
    /// Whether to use IPv6 or not.
    use_ipv6: bool,

    #[getset(set, get)]
    #[builder(private, default = "Box::new(DefaultExecCommand)")]
    /// Internal command executor to be used
    exec_command: Box<dyn ExecCommand>,
}

#[async_trait]
impl Iptables for DefaultIptables {
    async fn setup(&self, chain: &Chain) -> Result<()> {
        self.chain_setup(chain).await
    }

    async fn teardown(&self, chain: &Chain) -> Result<()> {
        self.chain_teardown(chain).await
    }

    fn set_ipv6(&mut self, value: bool) {
        self.set_use_ipv6(value);
    }
}

impl Debug for DefaultIptables {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Sandbox")
            .field("iptables_binary", self.iptables_binary())
            .field("ip6tables_binary", self.ip6tables_binary())
            .field("use_ipv6", &self.use_ipv6())
            .finish()
    }
}

#[async_trait]
trait ExecCommand: DynClone + Send + Sync {
    async fn output(&self, command: &mut Command) -> Result<Output> {
        command.output().await.context("run command")
    }
}

clone_trait_object!(ExecCommand);

#[derive(Clone, Default)]
/// DefaultExecCommand is a wrapper which can be used to execute a command in a standard way.
struct DefaultExecCommand;

impl ExecCommand for DefaultExecCommand {}

#[derive(Builder, Default, Debug, CopyGetters, Getters)]
#[builder(default, pattern = "owned", setter(into))]
/// Abstraction of an iptables chain.
pub struct Chain {
    #[get = "pub"]
    #[builder(default = r#""nat".to_string()"#)]
    /// Target table of the chain.
    table: String,

    #[get = "pub"]
    /// Name of the chain.
    name: String,

    #[get = "pub"]
    /// The chains to add the entry rule:
    entry_chains: Vec<String>,

    #[get = "pub"]
    // The rules that "point" to this chain.
    entry_rules: Vec<Vec<String>>,

    #[get = "pub"]
    /// The rules this chain contains.
    rules: Vec<Vec<String>>,

    #[get_copy = "pub"]
    /// Whether or not the entry rules should be prepended.
    prepend: bool,
}

impl DefaultIptables {
    /// Setup the provided `Chain` with respect of whether to use IPv6.
    async fn chain_setup(&self, chain: &Chain) -> Result<()> {
        // Create the chain if it not exists
        self.chain_ensure(chain).await.context("ensure chain")?;

        // Add the rules to the chain
        for rule in chain.rules() {
            self.rule_insert_unique(
                chain,
                &rule.iter().map(AsRef::as_ref).collect::<Vec<_>>(),
                false,
            )
            .await
            .context("insert rule")?;
        }

        // Add the entry rules to the entry chains
        for entry_chain_name in chain.entry_chains() {
            for rule in chain.entry_rules() {
                let mut new_rule = rule.to_vec();
                new_rule.push("-j".into());
                new_rule.push(chain.name().into());

                self.rule_insert_unique(
                    &ChainBuilder::default()
                        .name(entry_chain_name)
                        .build()
                        .context("build entry chain")?,
                    &new_rule.iter().map(AsRef::as_ref).collect::<Vec<_>>(),
                    chain.prepend(),
                )
                .await
                .context("insert entry chain rule")?;
            }
        }

        Ok(())
    }

    /// Idempotently delete a chain. It will not error if the chain doesn't exist. It will first
    /// delete all references to this chain in the `entry_chains`.
    async fn chain_teardown(&self, chain: &Chain) -> Result<()> {
        // This will succeed and create the chain if it does not exist. If the chain doesn't exist,
        // the next checks will fail.
        self.chain_clear(chain).await.context("clear chain")?;

        for entry_chain_name in chain.entry_chains() {
            let entry_chain = ChainBuilder::default()
                .name(entry_chain_name)
                .build()
                .context("build entry chain")?;

            for entry_chain_rule in self
                .rules_list(&entry_chain)
                .await
                .context("list chain rules")?
                .lines()
                // filter for rules in the correct target
                .filter(|x| x.strip_suffix(&format!("-j {}", chain.name())).is_some())
            {
                self.rule_delete(
                    &entry_chain,
                    &entry_chain_rule
                        .split_whitespace()
                        .filter(|x| !(x.starts_with('[') && x.ends_with(']'))) // filter out nftables prefixes "[0:0]"
                        .skip(2) // skip the rule prefix
                        .collect::<Vec<_>>(),
                )
                .await
                .context("delete rule")?;
            }
        }

        self.chain_delete(chain).await.context("delete chain")
    }

    /// Aadd a rule to a chain if it does not already exist. By default the rule is appended, unless `prepend` is true.
    async fn rule_insert_unique(&self, chain: &Chain, rule: &[&str], prepend: bool) -> Result<()> {
        trace!("Inserting unique rule");

        if !self.rule_exists(chain, rule).await {
            if prepend {
                self.rule_prepend(chain, rule)
                    .await
                    .context("prepend rule")?;
            } else {
                self.rule_append(chain, rule).await.context("append rule")?;
            }
        }

        Ok(())
    }

    /// Prepend a rule.
    async fn rule_prepend(&self, chain: &Chain, rule: &[&str]) -> Result<()> {
        trace!("Prepending rule");
        let mut args = vec!["-t", chain.table(), "-I", chain.name(), "1"];
        args.append(&mut rule.to_vec());
        self.run(&args).await?;
        Ok(())
    }

    /// Append a rule.
    async fn rule_append(&self, chain: &Chain, rule: &[&str]) -> Result<()> {
        trace!("Appending rule");
        let mut args = vec!["-t", chain.table(), "-A", chain.name()];
        args.append(&mut rule.to_vec());
        self.run(&args).await?;
        Ok(())
    }

    /// Delete the iptables rule in the specified `chain`. It does not error if the referring chain
    /// doesn't exist.
    async fn rule_delete(&self, chain: &Chain, rule: &[&str]) -> Result<()> {
        trace!("Deleting rule");
        if self.rule_exists(chain, rule).await {
            trace!("Rule exists, deleting now");
            let mut args = vec!["-t", chain.table(), "-D", chain.name()];
            args.append(&mut rule.to_vec());
            self.run(&args).await?;
        } else {
            trace!("Rule does not seem to exist");
        }
        Ok(())
    }

    /// Checks if given rulespec in specified if the provided `chain` exists.
    async fn rule_exists(&self, chain: &Chain, rule: &[&str]) -> bool {
        trace!("Checking if rule exists");
        let mut args = vec!["-t", chain.table(), "-C", chain.name()];
        args.append(&mut rule.to_vec());
        self.run(&args).await.is_ok()
    }

    /// Check whether the provided `chain` exists in the table and create if not.
    async fn chain_ensure(&self, chain: &Chain) -> Result<()> {
        trace!(
            "Ensuring iptables chain {} in table {}",
            chain.name(),
            chain.table()
        );

        if !self
            .chain_exists(chain)
            .await
            .context("check if chain exists")?
        {
            self.chain_new(chain).await.context("create new chain")?;
        }

        Ok(())
    }

    /// Create a new chain.
    async fn chain_new(&self, chain: &Chain) -> Result<()> {
        trace!(
            "Creating iptables chain {} in table {}",
            chain.name(),
            chain.table()
        );
        self.run(&["-t", chain.table(), "-N", chain.name()]).await?;
        Ok(())
    }

    /// Deletes the specified table/chain. It does not return an errors if the chain does not exist
    async fn chain_delete(&self, chain: &Chain) -> Result<()> {
        trace!(
            "Deleting iptables chain {} from table {}",
            chain.name(),
            chain.table()
        );

        if self
            .chain_exists(chain)
            .await
            .context("check if chain exists")?
        {
            self.run(&["-t", chain.table(), "-X", chain.name()]).await?;
        }
        Ok(())
    }

    // Clear the iptables rules in the specified table/chain. If the chain does not exist, a new one
    // will be created.
    async fn chain_clear(&self, chain: &Chain) -> Result<()> {
        trace!(
            "Clearing iptables chain {} in table {}",
            chain.name(),
            chain.table()
        );

        self.chain_ensure(chain).await.context("ensure chain")?;
        self.chain_flush(chain).await.context("flush chain")
    }

    // Clear the iptables rules in the specified table/chain. If the chain does not exist, a new one
    // will be created.
    async fn chain_flush(&self, chain: &Chain) -> Result<()> {
        trace!(
            "Flushing iptables chain {} in table {}",
            chain.name(),
            chain.table()
        );

        self.run(&["-t", chain.table(), "-F", chain.name()]).await?;
        Ok(())
    }

    /// Checks if the provided `chain` exists.
    async fn chain_exists(&self, chain: &Chain) -> Result<bool> {
        trace!(
            "Checking if iptables chain {} exists in table {}",
            chain.name(),
            chain.table()
        );
        Ok(self
            .chain_names(chain.table())
            .await
            .context("list chains")?
            .iter()
            .any(|chain_name| chain_name == chain.name()))
    }

    /// List all available itables chain names for the provided `table`.
    async fn chain_names(&self, table: &str) -> Result<Vec<String>> {
        trace!("Listing all chains in table {}", table);
        let output = self.run(&["-t", table, "-S"]).await?;

        // Iterate over rules to find all default (-P) and user-specified (-N) chains. Chains
        // definition always come before rules. Format is the following:
        // -P OUTPUT ACCEPT
        // -N Custom
        let mut chains = vec![];
        for line in output.lines() {
            match (line.strip_prefix("-P"), line.strip_prefix("-N")) {
                (Some(chain), _) | (_, Some(chain)) => chains.push(
                    chain
                        .trim()
                        .split_whitespace()
                        .next()
                        .with_context(|| format!("invalid chain output format: {}", line))?
                        .into(),
                ),
                _ => break,
            }
        }

        Ok(chains)
    }

    // List the rules in the provided `chain`.
    async fn rules_list(&self, chain: &Chain) -> Result<String> {
        trace!(
            "Listing rules for chain {} in table {}",
            chain.name(),
            chain.table()
        );
        self.run(&["-t", chain.table(), "-S", chain.name()]).await
    }

    /// Run an iptables command and retrieve its output.
    async fn run(&self, args: &[&str]) -> Result<String> {
        let binary = if self.use_ipv6() {
            self.ip6tables_binary()
        } else {
            self.iptables_binary()
        };
        trace!("Running: {} {}", binary.display(), args.join(" "));

        let output = self
            .exec_command()
            .output(Command::new(&binary).arg("--wait").args(args))
            .await
            .context("run iptables")?;

        if !output.status.success() {
            bail!(
                "command {} {} failed with error: {}",
                binary.display(),
                args.join(" "),
                String::from_utf8(output.stderr).context("convert stderr to string")?
            )
        }

        Ok(String::from_utf8(output.stdout)
            .context("convert stdout to string")?
            .trim()
            .into())
    }
}

impl Chain {
    pub const MARK_MASQ_CHAIN_NAME: &'static str = "CRI-HOSTPORT-MASQ";
    pub const SET_MARK_CHAIN_NAME: &'static str = "CRI-HOSTPORT-SETMARK";
    pub const TOP_LEVEL_DNAT_CHAIN_NAME: &'static str = "CRI-HOSTPORT-DNAT";

    /// Fill the DNAT rules for the chain.
    pub fn fill_dnat_rules(&mut self, port_mappings: &[&PortMapping], network: IpNetwork) {
        // Generate the dnat entry rules. We'll use multiport, but it ony accepts up to 15 rules,
        // so partition the list if needed.
        let protocol_ports = Self::ports_by_protocol(port_mappings);
        for (protocol, ports) in protocol_ports {
            for port_chunk in ports
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .chunks(15)
                .collect::<Vec<&[String]>>()
                .iter()
                .map(|chunk| chunk.join(","))
                .collect::<Vec<_>>()
            {
                self.entry_rules.push(
                    [
                        "-m",
                        "comment",
                        "--comment",
                        self.name(),
                        "-m",
                        "multiport",
                        "-p",
                        protocol,
                        "--destination-ports",
                        &port_chunk,
                    ]
                    .iter()
                    .map(ToString::to_string)
                    .collect(),
                );
            }
        }

        // For every entry, generate 3 rules:
        // - mark hairpin for masq
        // - mark localhost for masq (for v4)
        // - do DNAT
        // the ordering is important here; the mark rules must be first.
        for port_mapping in port_mappings {
            // If a HostIP is given, only process the entry if host and container address families
            // match and append it to the iptables rules.
            let mut base_rule = vec![
                "-p".into(),
                port_mapping.protocol().into(),
                "--dport".into(),
                port_mapping.host().port().to_string(),
            ];
            if !port_mapping.host().ip().is_unspecified() {
                base_rule.append(&mut vec!["-d".into(), port_mapping.host().ip().to_string()]);
            }

            // Add the hairpin rule.
            let mut hairpin_rule = base_rule.clone();
            hairpin_rule.append(&mut vec![
                "-s".into(),
                network.to_string(),
                "-j".into(),
                Self::SET_MARK_CHAIN_NAME.into(),
            ]);
            self.rules.push(hairpin_rule);

            // Add the localhost rule if necessary.
            if network.is_ipv4() {
                let mut local_rule = base_rule.clone();
                local_rule.append(&mut vec![
                    "-s".into(),
                    Ipv4Addr::LOCALHOST.to_string(),
                    "-j".into(),
                    Self::SET_MARK_CHAIN_NAME.into(),
                ]);
                self.rules.push(local_rule);
            }

            // Add the DNAT rule.
            let mut dnat_rule = base_rule.clone();
            dnat_rule.append(&mut vec![
                "-j".into(),
                "DNAT".into(),
                "--to-destination".into(),
                if network.is_ipv6() {
                    format!("[{}]:{}", network.ip(), port_mapping.container_port())
                } else {
                    format!("{}:{}", network.ip(), port_mapping.container_port())
                },
            ]);
            self.rules.push(dnat_rule)
        }
    }

    /// Group port numbers by protocols
    fn ports_by_protocol<'a>(port_mappings: &'a [&PortMapping]) -> BTreeMap<&'a str, Vec<u16>> {
        let mut result = BTreeMap::new();
        for port_mapping in port_mappings {
            result
                .entry(port_mapping.protocol().as_ref())
                .or_insert_with(Vec::new)
                .push(port_mapping.host().port());
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::cni::port::PortMappingBuilder;
    use std::{net::SocketAddr, os::unix::process::ExitStatusExt, process::ExitStatus, sync::Arc};
    use tokio::sync::RwLock;

    const LIST_CHAINS_OUTPUT: &str = r#"-P PREROUTING ACCEPT
-P INPUT ACCEPT
-P OUTPUT ACCEPT
-P POSTROUTING ACCEPT
-N CRI-HOSTPORT-DNAT
-N CRI-HOSTPORT-MASQ
-N CRI-HOSTPORT-SETMARK
-A PREROUTING -m addrtype --dst-type LOCAL -j CRI-HOSTPORT-DNAT
-A OUTPUT -m addrtype --dst-type LOCAL -j CRI-HOSTPORT-DNAT
-A POSTROUTING -m comment --comment portforward-requiring-masquerade -j CRI-HOSTPORT-MASQ
-A CRI-HOSTPORT-MASQ -m mark --mark 0x2000/0x2000 -j MASQUERADE
-A CRI-HOSTPORT-SETMARK -m comment --comment portforward-masquerade-mark -j MARK --set-xmark 0x2000/0x2000"#;

    const LIST_RULES_OUTPUT: &str = r#"-A POSTROUTING -m comment --comment portforward-requiring-masquerade -j CRI-HOSTPORT-MASQ
-A CRI-HOSTPORT-MASQ -m mark --mark 0x2000/0x2000 -j MASQUERADE
-A CRI-HOSTPORT-SETMARK -m comment --comment portforward-masquerade-mark -j MARK --set-xmark 0x2000/0x2000"#;

    const LIST_RULES_OUTPUT_NFTABLES: &str = r#"[0:0] -A POSTROUTING -m comment --comment portforward-requiring-masquerade -j CRI-HOSTPORT-MASQ
[0:1] -A CRI-HOSTPORT-MASQ -m mark --mark 0x2000/0x2000 -j MASQUERADE
[0:2] -A CRI-HOSTPORT-SETMARK -m comment --comment portforward-masquerade-mark -j MARK --set-xmark 0x2000/0x2000"#;

    #[derive(Clone, Debug, Getters, CopyGetters)]
    struct ExecCommandMock {
        output: Vec<Output>,
        call_index: Arc<RwLock<usize>>,
    }

    impl Default for ExecCommandMock {
        fn default() -> Self {
            Self {
                output: vec![],
                call_index: Arc::new(RwLock::new(0)),
            }
        }
    }

    #[async_trait]
    impl ExecCommand for ExecCommandMock {
        async fn output(&self, _: &mut Command) -> Result<Output> {
            let mut index = self.call_index.write().await;
            let output = self
                .output
                .get(*index)
                .with_context(|| format!("no call for index {}", *index))?;
            *index += 1;
            Ok(output.clone())
        }
    }

    impl ExecCommandMock {
        fn to_iptables(self) -> Result<DefaultIptables> {
            let mut iptables = DefaultIptablesBuilder::default()
                .iptables_binary("")
                .ip6tables_binary("")
                .build()?;

            iptables.set_exec_command(Box::new(self));

            Ok(iptables)
        }

        fn chain(&self) -> Result<Chain> {
            Ok(ChainBuilder::default()
                .name(Chain::TOP_LEVEL_DNAT_CHAIN_NAME)
                .build()?)
        }

        async fn add_call(&mut self, exit_code: i32, stdout: Option<&str>) {
            self.output.push(Output {
                status: ExitStatus::from_raw(exit_code),
                stdout: if let Some(stdout) = stdout {
                    stdout.as_bytes().to_vec()
                } else {
                    vec![]
                },

                stderr: vec![],
            });
        }

        async fn add_any_success(&mut self) {
            self.add_call(0, None).await;
        }

        async fn add_any_failure(&mut self) {
            self.add_call(1, None).await;
        }

        async fn mock_chain_exists(&mut self) {
            self.add_call(0, Some(LIST_CHAINS_OUTPUT)).await;
        }

        async fn add_chain_ensure(&mut self) {
            self.mock_chain_exists().await;
        }

        async fn add_rule_insert_unique(&mut self) {
            self.add_any_failure().await; // rule_exists
            self.add_any_success().await;
        }

        async fn add_chain_clear(&mut self) {
            self.add_chain_ensure().await;
            self.add_any_success().await; // chain_flush
        }

        async fn add_rule_delete(&mut self) {
            self.add_any_success().await; // rule_exists
            self.add_any_success().await;
        }

        async fn add_chain_delete(&mut self) {
            self.mock_chain_exists().await;
            self.add_any_success().await;
        }
    }

    #[tokio::test]
    async fn chain_teardown_success() -> Result<()> {
        let mut mock = ExecCommandMock::default();
        mock.add_chain_clear().await;

        mock.add_call(0, Some(LIST_RULES_OUTPUT)).await; // rules_list
        mock.add_rule_delete().await;
        mock.add_chain_delete().await;

        let chain = ChainBuilder::default()
            .name(Chain::MARK_MASQ_CHAIN_NAME)
            .entry_chains(vec!["MASQUERADE".into()])
            .build()?;

        let iptables = mock.to_iptables()?;
        iptables.chain_teardown(&chain).await
    }

    #[tokio::test]
    async fn chain_teardown_success_nftables() -> Result<()> {
        let mut mock = ExecCommandMock::default();
        mock.add_chain_clear().await;

        mock.add_call(0, Some(LIST_RULES_OUTPUT_NFTABLES)).await; // rules_list
        mock.add_rule_delete().await;
        mock.add_chain_delete().await;

        let chain = ChainBuilder::default()
            .name(Chain::MARK_MASQ_CHAIN_NAME)
            .entry_chains(vec!["MASQUERADE".into()])
            .build()?;

        let iptables = mock.to_iptables()?;
        iptables.chain_teardown(&chain).await
    }

    #[tokio::test]
    async fn chain_teardown_failure_chain_delete() -> Result<()> {
        let mut mock = ExecCommandMock::default();
        mock.add_chain_clear().await;

        mock.add_call(0, Some(LIST_RULES_OUTPUT)).await; // rules_list
        mock.add_rule_delete().await;

        mock.mock_chain_exists().await;
        mock.add_any_failure().await; // chain_delete

        let chain = ChainBuilder::default()
            .name(Chain::MARK_MASQ_CHAIN_NAME)
            .entry_chains(vec!["MASQUERADE".into()])
            .build()?;

        let iptables = mock.to_iptables()?;
        assert!(iptables.chain_teardown(&chain).await.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn chain_teardown_failure_rule_delete() -> Result<()> {
        let mut mock = ExecCommandMock::default();
        mock.add_chain_clear().await;

        mock.add_call(0, Some(LIST_RULES_OUTPUT)).await; // rules_list

        mock.add_any_success().await; // rule_exists
        mock.add_any_failure().await; // rule_delete

        let chain = ChainBuilder::default()
            .name(Chain::MARK_MASQ_CHAIN_NAME)
            .entry_chains(vec!["MASQUERADE".into()])
            .build()?;

        let iptables = mock.to_iptables()?;
        assert!(iptables.chain_teardown(&chain).await.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn chain_teardown_failure_rules_list() -> Result<()> {
        let mut mock = ExecCommandMock::default();
        mock.add_chain_clear().await;
        mock.add_any_failure().await; // rules_list

        let chain = ChainBuilder::default()
            .name(Chain::MARK_MASQ_CHAIN_NAME)
            .entry_chains(vec!["MASQUERADE".into()])
            .build()?;

        let iptables = mock.to_iptables()?;
        assert!(iptables.chain_teardown(&chain).await.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn chain_teardown_failure_chain_clear() -> Result<()> {
        let mut mock = ExecCommandMock::default();
        mock.add_any_failure().await; // chain_ensure

        let chain = ChainBuilder::default()
            .name(Chain::MARK_MASQ_CHAIN_NAME)
            .entry_chains(vec!["MASQUERADE".into()])
            .build()?;

        let iptables = mock.to_iptables()?;
        assert!(iptables.chain_teardown(&chain).await.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn chain_setup_success() -> Result<()> {
        let mut mock = ExecCommandMock::default();
        mock.add_chain_ensure().await;
        mock.add_rule_insert_unique().await; // 1
        mock.add_rule_insert_unique().await; // 2
        mock.add_rule_insert_unique().await; // 3

        let chain = ChainBuilder::default()
            .name(Chain::TOP_LEVEL_DNAT_CHAIN_NAME)
            .rules(vec![vec!["1".into()]])
            .entry_rules(vec![vec!["2".into()], vec!["3".into()]])
            .entry_chains(vec!["POSTROUTING".into()])
            .build()?;

        let iptables = mock.to_iptables()?;
        iptables.chain_setup(&chain).await
    }

    #[tokio::test]
    async fn chain_setup_failure_insert_unique_entry_chains() -> Result<()> {
        let mut mock = ExecCommandMock::default();
        mock.add_chain_ensure().await;
        mock.add_rule_insert_unique().await; // 1
        mock.add_rule_insert_unique().await; // 2

        mock.add_any_failure().await; // 3: rule_exists
        mock.add_any_failure().await; // 3: rule_prepend

        let chain = ChainBuilder::default()
            .name(Chain::TOP_LEVEL_DNAT_CHAIN_NAME)
            .rules(vec![vec!["1".into()]])
            .entry_rules(vec![vec!["2".into()], vec!["3".into()]])
            .entry_chains(vec!["POSTROUTING".into()])
            .build()?;

        let iptables = mock.to_iptables()?;
        assert!(iptables.chain_setup(&chain).await.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn chain_setup_failure_insert_unique_rules() -> Result<()> {
        let mut mock = ExecCommandMock::default();
        mock.add_chain_ensure().await;
        mock.add_any_failure().await; // 1: rule_exists
        mock.add_any_failure().await; // 1: rule_prepend

        let chain = ChainBuilder::default()
            .name(Chain::TOP_LEVEL_DNAT_CHAIN_NAME)
            .rules(vec![vec!["1".into()]])
            .entry_chains(vec!["POSTROUTING".into()])
            .build()?;

        let iptables = mock.to_iptables()?;
        assert!(iptables.chain_setup(&chain).await.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn rule_insert_unique_success_not_exists_prepend() -> Result<()> {
        let mut mock = ExecCommandMock::default();
        mock.add_rule_insert_unique().await;

        let chain = mock.chain()?;
        let iptables = mock.to_iptables()?;
        iptables.rule_insert_unique(&chain, &[], true).await
    }

    #[tokio::test]
    async fn rule_insert_unique_success_not_exists_append() -> Result<()> {
        let mut mock = ExecCommandMock::default();
        mock.add_rule_insert_unique().await;

        let chain = mock.chain()?;
        let iptables = mock.to_iptables()?;
        iptables.rule_insert_unique(&chain, &[], false).await
    }

    #[tokio::test]
    async fn rule_insert_unique_success_exists() -> Result<()> {
        let mut mock = ExecCommandMock::default();
        mock.add_any_success().await; // rule_exists

        let chain = mock.chain()?;
        let iptables = mock.to_iptables()?;
        iptables.rule_insert_unique(&chain, &[], false).await
    }

    #[tokio::test]
    async fn rule_insert_unique_failure() -> Result<()> {
        let mut mock = ExecCommandMock::default();
        mock.add_any_failure().await; // rule_exists
        mock.add_any_failure().await; // rule_prepend

        let chain = mock.chain()?;
        let iptables = mock.to_iptables()?;
        assert!(iptables
            .rule_insert_unique(&chain, &[], true)
            .await
            .is_err());
        Ok(())
    }

    #[tokio::test]
    async fn rule_delete_success_exists() -> Result<()> {
        let mut mock = ExecCommandMock::default();
        mock.add_any_success().await; // rule_exists
        mock.add_any_success().await;

        let chain = mock.chain()?;
        let iptables = mock.to_iptables()?;
        iptables.rule_delete(&chain, &[]).await
    }

    #[tokio::test]
    async fn rule_delete_success_not_exists() -> Result<()> {
        let mut mock = ExecCommandMock::default();
        mock.add_any_failure().await; // rule_exists

        let chain = mock.chain()?;
        let iptables = mock.to_iptables()?;
        iptables.rule_delete(&chain, &[]).await
    }

    #[tokio::test]
    async fn rule_delete_failure() -> Result<()> {
        let mut mock = ExecCommandMock::default();
        mock.add_any_success().await; // rule_exists
        mock.add_any_failure().await;

        let chain = mock.chain()?;
        let iptables = mock.to_iptables()?;
        assert!(iptables.rule_delete(&chain, &[]).await.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn chain_delete_success_exists() -> Result<()> {
        let mut mock = ExecCommandMock::default();
        mock.add_chain_delete().await;

        let chain = mock.chain()?;
        let iptables = mock.to_iptables()?;
        iptables.chain_delete(&chain).await
    }

    #[tokio::test]
    async fn chain_delete_success_not_exists() -> Result<()> {
        let mut mock = ExecCommandMock::default();
        mock.mock_chain_exists().await;
        mock.add_any_success().await;

        let chain = ChainBuilder::default().name("").build()?;
        let iptables = mock.to_iptables()?;
        iptables.chain_delete(&chain).await
    }

    #[tokio::test]
    async fn chain_delete_failure_exists() -> Result<()> {
        let mut mock = ExecCommandMock::default();
        mock.mock_chain_exists().await;
        mock.add_any_failure().await;

        let chain = mock.chain()?;
        let iptables = mock.to_iptables()?;
        assert!(iptables.chain_delete(&chain).await.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn chain_delete_failure_exists_failed() -> Result<()> {
        let mut mock = ExecCommandMock::default();
        mock.add_any_failure().await;

        let chain = mock.chain()?;
        let iptables = mock.to_iptables()?;
        assert!(iptables.chain_delete(&chain).await.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn chain_clear_succes() -> Result<()> {
        let mut mock = ExecCommandMock::default();
        mock.add_chain_clear().await;

        let chain = mock.chain()?;
        let iptables = mock.to_iptables()?;
        iptables.chain_clear(&chain).await
    }

    #[tokio::test]
    async fn chain_clear_failure_ensure() -> Result<()> {
        let mut mock = ExecCommandMock::default();
        mock.add_any_failure().await; // chain_ensure

        let chain = mock.chain()?;
        let iptables = mock.to_iptables()?;
        assert!(iptables.chain_clear(&chain).await.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn chain_clear_failure_flush() -> Result<()> {
        let mut mock = ExecCommandMock::default();
        mock.add_chain_ensure().await;
        mock.add_any_failure().await; // chain_flush

        let chain = mock.chain()?;
        let iptables = mock.to_iptables()?;
        assert!(iptables.chain_clear(&chain).await.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn chain_ensure_succes_exists() -> Result<()> {
        let mut mock = ExecCommandMock::default();
        mock.add_chain_ensure().await;

        let chain = mock.chain()?;
        let iptables = mock.to_iptables()?;
        iptables.chain_ensure(&chain).await
    }

    #[tokio::test]
    async fn chain_ensure_succes_not_exists() -> Result<()> {
        let mut mock = ExecCommandMock::default();
        mock.add_chain_ensure().await;
        mock.add_any_success().await; // chain_new

        let chain = ChainBuilder::default().name("").build()?;
        let iptables = mock.to_iptables()?;
        iptables.chain_ensure(&chain).await
    }

    #[tokio::test]
    async fn chain_ensure_failure() -> Result<()> {
        let mut mock = ExecCommandMock::default();
        mock.add_chain_ensure().await;
        mock.add_any_failure().await; // chain_new

        let chain = ChainBuilder::default().name("").build()?;
        let iptables = mock.to_iptables()?;
        assert!(iptables.chain_ensure(&chain).await.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn chain_exists_success_true() -> Result<()> {
        let mut mock = ExecCommandMock::default();
        mock.mock_chain_exists().await;

        let chain = mock.chain()?;
        let iptables = mock.to_iptables()?;
        let exists = iptables.chain_exists(&chain).await?;
        assert!(exists);
        Ok(())
    }

    #[tokio::test]
    async fn chain_exists_success_false() -> Result<()> {
        let mut mock = ExecCommandMock::default();
        mock.mock_chain_exists().await;

        let chain = ChainBuilder::default().name("").build()?;
        let iptables = mock.to_iptables()?;
        let exists = iptables.chain_exists(&chain).await?;
        assert!(!exists);
        Ok(())
    }

    #[tokio::test]
    async fn chain_exists_failure_invalid_format() -> Result<()> {
        let mut mock = ExecCommandMock::default();
        mock.add_call(0, Some("-P")).await;

        let chain = ChainBuilder::default().build()?;
        let iptables = mock.to_iptables()?;
        assert!(iptables.chain_exists(&chain).await.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn run_success_mocked() -> Result<()> {
        let mut mock = ExecCommandMock::default();
        mock.add_call(0, None).await;

        let iptables = mock.to_iptables()?;
        let output = iptables.run(&[]).await?;
        assert!(output.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn run_failure_mocked() -> Result<()> {
        let mut mock = ExecCommandMock::default();
        mock.add_any_failure().await;

        let iptables = mock.to_iptables()?;
        assert!(iptables.run(&[]).await.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn run_success_ipv4() -> Result<()> {
        let echo = which::which("echo")?;
        let iptables = DefaultIptablesBuilder::default()
            .iptables_binary(&echo)
            .ip6tables_binary("wrong")
            .build()?;

        let output = iptables.run(&["hello", "world"]).await?;

        assert_eq!(output, "--wait hello world");
        Ok(())
    }

    #[tokio::test]
    async fn run_success_ipv6() -> Result<()> {
        let echo = which::which("echo")?;
        let iptables = DefaultIptablesBuilder::default()
            .iptables_binary("wrong")
            .ip6tables_binary(&echo)
            .use_ipv6(true)
            .build()?;

        let output = iptables.run(&["hello", "world"]).await?;

        assert_eq!(output, "--wait hello world");
        Ok(())
    }

    #[tokio::test]
    async fn run_failure_invalid_binary() -> Result<()> {
        let iptables = DefaultIptablesBuilder::default()
            .iptables_binary("wrong")
            .ip6tables_binary("wrong")
            .build()?;

        assert!(iptables.run(&[]).await.is_err());
        Ok(())
    }

    #[test]
    fn fill_dnat_rules() -> Result<()> {
        let mut chain = ChainBuilder::default().name("name").build()?;
        chain.fill_dnat_rules(
            &[
                &PortMappingBuilder::default()
                    .host("127.0.0.1:8080".parse::<SocketAddr>()?)
                    .container_port(8080u16)
                    .protocol("tcp")
                    .build()?,
                &PortMappingBuilder::default()
                    .host("127.0.0.1:8081".parse::<SocketAddr>()?)
                    .container_port(8081u16)
                    .protocol("udp")
                    .build()?,
                &PortMappingBuilder::default()
                    .host("0.0.0.0:30443".parse::<SocketAddr>()?)
                    .container_port(1234u16)
                    .protocol("sctp")
                    .build()?,
                &PortMappingBuilder::default()
                    .host("[::1]:6000".parse::<SocketAddr>()?)
                    .container_port(6001u16)
                    .protocol("tcp")
                    .build()?,
            ],
            IpNetwork::V4("10.0.0.1/16".parse()?),
        );

        assert_eq!(chain.table(), "nat");
        assert_eq!(chain.name(), "name");
        assert!(chain.entry_chains().is_empty());
        assert!(!chain.prepend());
        assert_eq!(
            chain
                .entry_rules()
                .iter()
                .map(|x| x.join(" "))
                .collect::<Vec<_>>()
                .join("\n"),
            r#"-m comment --comment name -m multiport -p sctp --destination-ports 30443
-m comment --comment name -m multiport -p tcp --destination-ports 8080,6000
-m comment --comment name -m multiport -p udp --destination-ports 8081"#,
        );
        assert_eq!(
            chain
                .rules()
                .iter()
                .map(|x| x.join(" "))
                .collect::<Vec<_>>()
                .join("\n"),
            r#"-p tcp --dport 8080 -d 127.0.0.1 -s 10.0.0.1/16 -j CRI-HOSTPORT-SETMARK
-p tcp --dport 8080 -d 127.0.0.1 -s 127.0.0.1 -j CRI-HOSTPORT-SETMARK
-p tcp --dport 8080 -d 127.0.0.1 -j DNAT --to-destination 10.0.0.1:8080
-p udp --dport 8081 -d 127.0.0.1 -s 10.0.0.1/16 -j CRI-HOSTPORT-SETMARK
-p udp --dport 8081 -d 127.0.0.1 -s 127.0.0.1 -j CRI-HOSTPORT-SETMARK
-p udp --dport 8081 -d 127.0.0.1 -j DNAT --to-destination 10.0.0.1:8081
-p sctp --dport 30443 -s 10.0.0.1/16 -j CRI-HOSTPORT-SETMARK
-p sctp --dport 30443 -s 127.0.0.1 -j CRI-HOSTPORT-SETMARK
-p sctp --dport 30443 -j DNAT --to-destination 10.0.0.1:1234
-p tcp --dport 6000 -d ::1 -s 10.0.0.1/16 -j CRI-HOSTPORT-SETMARK
-p tcp --dport 6000 -d ::1 -s 127.0.0.1 -j CRI-HOSTPORT-SETMARK
-p tcp --dport 6000 -d ::1 -j DNAT --to-destination 10.0.0.1:6001"#,
        );

        Ok(())
    }
}
