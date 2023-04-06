#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

// pink_extension is short for Phala ink! extension
use pink_extension as pink;

#[pink::contract(env=PinkEnvironment)]
mod query_indexer {
    use super::pink;
    use ink::env::debug_println;
    use ink::prelude::{string::String, format, vec};
    use pink::{http_post, PinkEnvironment};
    use scale::{Decode, Encode};
    use serde::{Deserialize};

    // you have to use crates with `no_std` support in contract.
    use serde_json_core;

    #[derive(Debug, PartialEq, Eq, Encode, Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        HttpRequestFailed,
        InvalidResponseBody,
    }

    /// Type alias for the contract's result type.
    pub type Result<T> = core::result::Result<T, Error>;

    #[ink(storage)]
    pub struct QueryIndexer {
        url: String,
    }

    #[derive(Deserialize, Encode, Clone, Debug, PartialEq)]
    struct IndexerRewardResponse<'a> {
        #[serde(borrow)]
        data: IndexerRewardData<'a>,
    }

    #[derive(Deserialize, Encode, Clone, Debug, PartialEq)]
    #[allow(non_snake_case)]
    struct IndexerRewardData<'a> {
        #[serde(borrow)]
        developerRewards: DeveloperRewards<'a>,
    }

    #[derive(Deserialize, Encode, Clone, Debug, PartialEq)]
    struct DeveloperRewards<'a> {
        #[serde(borrow)]
        nodes: [DeveloperRewardNode<'a>; 1],
    }

    #[derive(Deserialize, Encode, Clone, Debug, PartialEq)]
    struct DeveloperRewardNode<'a> {
        amount: &'a str,
        era: &'a str,
    }


    impl QueryIndexer {

        #[ink(constructor)]
        pub fn new(url: String) -> Self {
            Self { url }
        }

        #[ink(constructor)]
        pub fn default() -> Self {
            Self { url: String::from("https://api.subquery.network/sq/GuiGou12358/lucky-shibuya-v0_1_0/") }
        }

        #[ink(message)]
        pub fn get_url(&self) -> Result<String> {
            Ok(self.url.clone())
        }

        #[ink(message)]
        pub fn get_developer_rewards(&self, era: u16) -> Result<String> {

            let headers = vec![
                ("Content-Type".into(), "application/json".into()),
                ("Accept".into(), "application/json".into())
            ];
            //  {"query" : "query {developerRewards (filter: { era: { equalTo: \"{}\" } }){nodes {amount, era}}}"}
            let body = format!(
                r#"{{"query" : "query {{developerRewards (filter: {{ era: {{ equalTo: \"{}\" }} }}){{nodes {{amount, era}}}}}}"}}"#,
                era
            );
            debug_println!("body: {}", body);

            let resp = http_post!(
                self.url.clone(),
                body,
                headers
            );
            debug_println!("status code {}", resp.status_code);
            debug_println!("body {}", std::str::from_utf8(&resp.body).unwrap());

            if resp.status_code != 200 {
                return Err(Error::HttpRequestFailed);
            }

            let result: IndexerRewardResponse = serde_json_core::from_slice(&resp.body)
                .or(Err(Error::InvalidResponseBody))?
                .0;
            Ok(String::from(result.data.developerRewards.nodes[0].amount))
        }


    }

    #[cfg(test)]
    mod tests {

        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        #[ink::test]
        fn get_developer_rewards() {
            // when your contract is really deployed, the Phala Worker will do the HTTP requests
            // mock is needed for local test
            pink_extension_runtime::mock_ext::mock_all_ext();

            let url = "https://api.subquery.network/sq/GuiGou12358/lucky-shibuya-v0_1_0/";

            let query_indexer = QueryIndexer::new(url.to_string());
            let era = 2800;
            let res = query_indexer.get_developer_rewards(era);

            assert!(res.is_ok());

            let r = res.unwrap();

            // run with `cargo +nightly test -- --nocapture` to see the following output
            println!("Era {} Developer rewards {}", era, r);

        }
    }
}
