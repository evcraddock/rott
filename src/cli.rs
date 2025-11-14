use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "rott")]
#[command(about = "Brain ROTT - Link management system", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Create a new resource
    Create {
        #[command(subcommand)]
        resource: CreateCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum CreateCommands {
    /// Create a new link from a URL
    Link {
        /// The URL to create a link for
        url: String,

        /// Tags to add to the link (comma-separated)
        #[arg(long, value_delimiter = ',')]
        tags: Option<Vec<String>>,

        /// Silent mode - suppress all output messages
        #[arg(short, long)]
        silent: bool,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_no_args() {
        let cli = Cli::try_parse_from(vec!["rott"]).unwrap();
        assert!(cli.command.is_none());
    }

    #[test]
    fn test_create_link_basic() {
        let cli = Cli::try_parse_from(vec!["rott", "create", "link", "https://example.com"]).unwrap();

        match cli.command {
            Some(Commands::Create { resource: CreateCommands::Link { url, tags, silent } }) => {
                assert_eq!(url, "https://example.com");
                assert!(tags.is_none());
                assert_eq!(silent, false);
            }
            _ => panic!("Expected Create Link command"),
        }
    }

    #[test]
    fn test_create_link_with_tags() {
        let cli = Cli::try_parse_from(vec![
            "rott",
            "create",
            "link",
            "https://example.com",
            "--tags",
            "rust,programming,cli"
        ]).unwrap();

        match cli.command {
            Some(Commands::Create { resource: CreateCommands::Link { url, tags, silent } }) => {
                assert_eq!(url, "https://example.com");
                assert_eq!(tags, Some(vec![
                    "rust".to_string(),
                    "programming".to_string(),
                    "cli".to_string()
                ]));
                assert_eq!(silent, false);
            }
            _ => panic!("Expected Create Link command"),
        }
    }

    #[test]
    fn test_create_link_with_single_tag() {
        let cli = Cli::try_parse_from(vec![
            "rott",
            "create",
            "link",
            "https://example.com",
            "--tags",
            "rust"
        ]).unwrap();

        match cli.command {
            Some(Commands::Create { resource: CreateCommands::Link { url, tags, silent } }) => {
                assert_eq!(url, "https://example.com");
                assert_eq!(tags, Some(vec!["rust".to_string()]));
                assert_eq!(silent, false);
            }
            _ => panic!("Expected Create Link command"),
        }
    }

    #[test]
    fn test_create_link_with_silent_flag() {
        let cli = Cli::try_parse_from(vec![
            "rott",
            "create",
            "link",
            "https://example.com",
            "--silent"
        ]).unwrap();

        match cli.command {
            Some(Commands::Create { resource: CreateCommands::Link { url, tags, silent } }) => {
                assert_eq!(url, "https://example.com");
                assert!(tags.is_none());
                assert_eq!(silent, true);
            }
            _ => panic!("Expected Create Link command"),
        }
    }

    #[test]
    fn test_create_link_with_silent_short_flag() {
        let cli = Cli::try_parse_from(vec![
            "rott",
            "create",
            "link",
            "https://example.com",
            "-s"
        ]).unwrap();

        match cli.command {
            Some(Commands::Create { resource: CreateCommands::Link { url, tags, silent } }) => {
                assert_eq!(url, "https://example.com");
                assert!(tags.is_none());
                assert_eq!(silent, true);
            }
            _ => panic!("Expected Create Link command"),
        }
    }

    #[test]
    fn test_create_link_with_silent_and_tags() {
        let cli = Cli::try_parse_from(vec![
            "rott",
            "create",
            "link",
            "https://example.com",
            "--tags",
            "rust,cli",
            "--silent"
        ]).unwrap();

        match cli.command {
            Some(Commands::Create { resource: CreateCommands::Link { url, tags, silent } }) => {
                assert_eq!(url, "https://example.com");
                assert_eq!(tags, Some(vec!["rust".to_string(), "cli".to_string()]));
                assert_eq!(silent, true);
            }
            _ => panic!("Expected Create Link command"),
        }
    }
}
