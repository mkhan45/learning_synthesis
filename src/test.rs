#[cfg(test)]
use crate::enumerative::top_down_vsa;
#[cfg(test)]
use crate::vsa::Lit::{self, *};

#[cfg(test)]
fn test_gen(examples: Vec<(Lit, Lit)>, tests: Vec<(Lit, Lit)>) {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let res = top_down_vsa(&examples);
        println!("{}, size = {}", res, res.size());

        for (inp, out) in tests {
            let evaled = res.eval(&inp);
            let evaled = match (&inp, &evaled) {
                (Lit::StringConst(s), Lit::LocEnd) => Lit::LocConst(s.len()),
                _ => evaled,
            };
            assert_eq!(evaled, out);
        }

        tx.send(()).unwrap();
    });

    if let Err(_) = rx.recv_timeout(std::time::Duration::from_secs(12)) {
        panic!("timeout");
    }
}

macro_rules! test {
    ($name:ident, $out_mapper:expr, $($inp:expr => $out:expr),+; $($test_inp:expr => $test_out:expr),+) => {
        #[test]
        fn $name() {
            test_gen(vec![
                $(
                    (
                        Lit::StringConst($inp.to_string()),
                        $out_mapper($out),
                    ),
                )+
            ], vec![
                $(
                    (
                        Lit::StringConst($test_inp.to_string()),
                        $out_mapper($test_out),
                    ),
                )+
            ]);
        }
    };
}

// Run these with cargo test --release -- --nocapture to see the output

test!(
    test_duet_date,
    |s: &str| StringConst(s.to_string()),
    "01/15/2013" => "01/2013",
    "03/07/2011" => "03/2011",
    "05/09/2009" => "05/2009";

    "01/02/03" => "01/03",
    "09/02/07" => "09/07"
);

test!(
    test_duet_numbers,
    |s: &str| StringConst(s.to_string()),
    "I have 17 cookies" => "17",
    "Give me at least 3 cookies" => "3",
    "This number is 489" => "489";

    "A string with the number 54234564 in the middle" => "54234564",
    "36" => "36",
    "Number at the end 74" => "74"
);

test!(
    test_duet_multiple_numbers,
    |s: &str| StringConst(s.to_string()),
    "This string has more than 1 number or 2 it has 3" => "2",
    "i want 56 the 74 second" => "74",
    "this one has 3 digit number at the end 698" => "698",
    "74 55 66" => "55";

    "aaaaaaaaaaaa 54 36 97 aaaaa" => "36",
    "testcases 33 are 45 hard" => "45"
);

test!(
    test_duet_abbrev,
    |s: &str| StringConst(s.to_string()),
    "First Last" => "F.L.",
    "Abc Defgh" => "A.D.",
    "Someone Name" => "S.N.";

    "Another Name" => "A.N."
);

test!(
    test_duet_model_no,
    |s: &str| StringConst(s.to_string()),
    "Tire Pressure ABC123873 Monitor" => "ABC123873",
    "ABC849999999021 Oil Life gauge" => "ABC849999999021";

    " Air conditioner GHF211 maintenance" => "GHF211"
);

test!(
    test_duet_url,
    |s: &str| StringConst(s.to_string()),
    "http://www.example.com" => "example",
    "https://www.apple.com/uk/mac" => "apple";

    "https://www.google.com" => "google"
);

// TODO: this one is much harder,
// probably need a new regex
// test_duet_str!(
//     test_duet_url,
//     "http://www.example.com" => "example",
//     "https://apple.com/uk/mac" => "apple";

//     "https://www.google.com" => "google"
// );

// TODO: for long strings, probably gotta use middle
// out so that the concat witness function isnt massive
//
// probably can use string length as a heuristic for when
// to switch from normal witness to fancier cut
//
// What's the difference between a normal witness function and a cut?
test!(
    test_delete_between,
    |s: &str| StringConst(s.to_string()),
    "short /no/ line" => "short  line",
    "aa/aa/aa" => "aaaa";

    "remove /this/" => "remove "
);

test!(
    test_duet_money,
    |s: &str| StringConst(s.to_string()),
    "USD.EUR<IDEALPRO,CASH,EUR>" => "EUR",
    "USD.EUR<IDEALPRO,CASH,USD>" => "USD";

    "KOR.JPN<IDEALPRO,CASH,WON>" => "WON",
    "USD.EUR<IDEALPRO,CASH,JPY>" => "JPY",
    "USD.KOR<IDEALPRO,CASH,GBP>" => "GBP"
);

test!(
    test_json,
    |s: &str| StringConst(s.to_string()),
    "one 1" => "{one: 1}",
    "three aaa" => "{three: aaa}";

    "two second example" => "{two: second example}",
    "four fourth example" => "{four: fourth example}"
);

test!(
    test_append,
    |s: &str| StringConst(s.to_string()),
    "Hello" => "Hello World",
    "Goodbye" => "Goodbye World";

    "B" => "B World"
);

test!(
    test_country_abbr,
    |s: &str| StringConst(s.to_string()),
    "Mining US" => "Mining",
    "Soybean Farming CAN" => "Soybean Farming";

    "Mining" => "Mining",
    "Soybean Farming" => "Soybean Farming",
    "Oil Extraction US" => "Oil Extraction",
    "Quarrying EU" => "Quarrying"
);

// Note: this test doesnt work with multiple numbers before the .
test!(
    test_version_no,
    |s: &str| StringConst(s.to_string()),
    "Red Hat Enterprise AS 4 <3.5-78.0.13.ELlargesmp>" => "3.5",
    "Microsoft Windows XP Win2008R 6.1.7601" => "6.1";

    "Linux Linux 2.6 Linux" => "2.6",
    "AIX 5.1" => "5.1",
    "VMware ESX Server 3.5.0 build-110268" => "3.5"
);

test!(
    test_rle_snippet,
    |s: &str| StringConst(s.to_string()),
    "aaabcdefg" => "aaa",
    "bcdefg" => "b",
    "eefg" => "ee";

    "sssss" => "sssss",
    "opasdf" => "o"
);

test! {
    test_length,
    LocConst,
    "a" => 1,
    "abcdefg" => 7;

    "abc" => 3,
    "1234567890" => 10
}

// temp to test tests
// test! {
//     test_find_length,
//     BoolConst,
//     "abcdef" => true,
//     "ABCDEF" => true,
//     "abcdefg" => false,
//     "def" => false,
//     "another string" => false,
//     "hijasdf" => true;

//     "abc" => false,
//     "1234567890" => false
// }

// test! {
//     test_starts_with,
//     BoolConst,
//     "abc" => true,
//     "acab" => true,
//     "something" => false,
//     "not a" => false;

//     "abc" => true,
//     "1234567890" => false
// }
