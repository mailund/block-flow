use ::block_macros::*;

#[test]
fn init_params_contract_deps_collects_and_skips() {
    mod case_trade_types_glob {
        use super::*;
        use ::block_traits::ContractDeps;
        use ::trade_types::*;

        #[init_params]
        #[allow(dead_code)]
        struct Params {
            a: Contract,
            b: Option<Contract>,
            c: Vec<Contract>,
            d: Option<Vec<Contract>>,
            #[no_contract_deps]
            skip1: Contract,
            #[no_contract_deps]
            skip2: Option<Contract>,
            #[no_contract_deps]
            skip3: Vec<Contract>,
            #[no_contract_deps]
            skip4: Option<Vec<Contract>>,
            other: u32,
        }

        pub fn run() {
            let a = Contract::new("A");
            let b = Contract::new("B");
            let c1 = Contract::new("C1");
            let c2 = Contract::new("C2");
            let d1 = Contract::new("D1");
            let d2 = Contract::new("D2");
            let skip1 = Contract::new("SKIP1");
            let skip2 = Contract::new("SKIP2");
            let skip3 = Contract::new("SKIP3");
            let skip4 = Contract::new("SKIP4");

            let p = Params {
                a: a.clone(),
                b: Some(b.clone()),
                c: vec![c1.clone(), c2.clone()],
                d: Some(vec![d1.clone(), d2.clone()]),
                skip1,
                skip2: Some(skip2),
                skip3: vec![skip3],
                skip4: Some(vec![skip4]),
                other: 123,
            };

            let got = p.contract_deps();
            assert_eq!(got, vec![a, b, c1, c2, d1, d2]);
        }
    }

    mod case_fully_qualified {
        use super::*;
        use ::block_traits::ContractDeps;

        #[init_params]
        #[allow(dead_code)]
        struct Params {
            a: ::trade_types::Contract,
            b: Option<::trade_types::Contract>,
            c: Vec<::trade_types::Contract>,
            d: Option<Vec<::trade_types::Contract>>,
            #[no_contract_deps]
            skip1: ::trade_types::Contract,
            #[no_contract_deps]
            skip2: Option<::trade_types::Contract>,
            #[no_contract_deps]
            skip3: Vec<::trade_types::Contract>,
            #[no_contract_deps]
            skip4: Option<Vec<::trade_types::Contract>>,
            other: u32,
        }

        pub fn run() {
            let a = ::trade_types::Contract::new("A");
            let b = ::trade_types::Contract::new("B");
            let c1 = ::trade_types::Contract::new("C1");
            let c2 = ::trade_types::Contract::new("C2");
            let d1 = ::trade_types::Contract::new("D1");
            let d2 = ::trade_types::Contract::new("D2");
            let skip1 = ::trade_types::Contract::new("SKIP1");
            let skip2 = ::trade_types::Contract::new("SKIP2");
            let skip3 = ::trade_types::Contract::new("SKIP3");
            let skip4 = ::trade_types::Contract::new("SKIP4");

            let p = Params {
                a: a.clone(),
                b: Some(b.clone()),
                c: vec![c1.clone(), c2.clone()],
                d: Some(vec![d1.clone(), d2.clone()]),
                skip1,
                skip2: Some(skip2),
                skip3: vec![skip3],
                skip4: Some(vec![skip4]),
                other: 456,
            };

            let got = p.contract_deps();
            assert_eq!(got, vec![a, b, c1, c2, d1, d2]);
        }
    }

    case_trade_types_glob::run();
    case_fully_qualified::run();
}
