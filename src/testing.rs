use scoped_threadpool::Scope;
use serde::Deserialize;

use crate::chess::{ab::SearchSettings, board::Board};

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestSuite {
    description: String,
    test_cases: Vec<TestCase>,
}

#[derive(Debug, Deserialize)]
pub struct TestCase {
    start: TestPosition,
    expected: Vec<TestMove>,
}

impl TestCase {
    #[allow(dead_code)]
    pub fn start(&self) -> &TestPosition {
        &self.start
    }

    #[allow(dead_code)]
    pub fn moves(&self) -> &[TestMove] {
        self.expected.as_ref()
    }
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct TestPosition {
    description: String,
    fen: String,
}

impl TestPosition {
    #[allow(dead_code)]
    pub fn fen(&self) -> &str {
        self.fen.as_ref()
    }

    #[allow(dead_code)]
    pub fn description(&self) -> &str {
        self.description.as_ref()
    }
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct TestMove {
    #[serde(alias = "move")]
    mv: String,
    fen: String,
}

impl TestMove {
    #[allow(dead_code)]
    pub fn mv(&self) -> &str {
        self.mv.as_ref()
    }

    #[allow(dead_code)]
    pub fn fen(&self) -> &str {
        self.fen.as_ref()
    }
}

impl TestSuite {
    #[allow(dead_code)]
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    #[allow(dead_code)]
    pub fn description(&self) -> &str {
        self.description.as_ref()
    }

    #[allow(dead_code)]
    pub fn test_cases(&self) -> &[TestCase] {
        self.test_cases.as_ref()
    }
}

#[cfg(test)]
mod test {
    use std::{
        fs::File,
        io::{BufRead, BufReader},
    };

    use crate::chess::{ab::SearchSettings, board::Board};

    use super::{test_with_epd, TestSuite};

    fn run_test(file: &str) {
        let suite = TestSuite::from_json(&std::fs::read_to_string(file).unwrap()).unwrap();

        eprintln!("starting test suite {}", suite.description());
        for case in suite.test_cases() {
            eprintln!("running test case {}", case.start().description());
            let b = Board::from_fen(case.start().fen()).unwrap();
            eprintln!("starting pos {}:\n{}", case.start().fen(), b);

            let num_calc_moves = b.legal_moves().count();
            let num_expected_moves = case.moves().len();
            if num_calc_moves != num_expected_moves {
                for m in b.legal_moves() {
                    eprintln!(" ==> {}", m);
                }
            }
            assert_eq!(num_calc_moves, num_expected_moves);
        }
    }

    #[test]
    fn test_castling() {
        let path = "testcases/castling.json";
        run_test(path);
    }

    #[test]
    fn test_famous() {
        let path = "testcases/famous.json";
        run_test(path);
    }

    #[test]
    fn test_pawns() {
        let path = "testcases/pawns.json";
        run_test(path);
    }

    #[test]
    fn test_promotions() {
        let path = "testcases/promotions.json";
        run_test(path);
    }

    #[test]
    fn test_standard() {
        let path = "testcases/standard.json";
        run_test(path);
    }

    #[test]
    fn test_taxing() {
        let path = "testcases/taxing.json";
        run_test(path);
    }

    extern crate test;
    use test::Bencher;

    #[allow(soft_unstable)]
    #[bench]
    fn bench_movegen(b: &mut Bencher) {
        let settings = SearchSettings::divide(3);
        let board =
            Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
        b.iter(|| {
            board.alphabeta(&settings, true);
        });
    }

    const MAX_DEPTH: u32 = 3;
    #[test]
    fn test_depth_many_up_to() {
        let file = "testcases/depths.epd";
        let lines = BufReader::new(File::open(file).unwrap()).lines();

        let file2 = "testcases/fischer.epd";
        let _lines2 = BufReader::new(File::open(file2).unwrap()).lines();
        let mut tp = scoped_threadpool::Pool::new(
            std::thread::available_parallelism()
                .unwrap()
                .try_into()
                .map(|x: usize| (x * 20).try_into().unwrap())
                .unwrap(),
        );

        tp.scoped(|scope| {
            for line in lines {
                test_with_epd(scope, &line.unwrap(), MAX_DEPTH);
            }
        });
    }
}

#[allow(dead_code)]
fn test_with_epd(scope: &Scope, epd: &str, max: u32) {
    let mut splits = epd.split(";");
    let fen = splits.nth(0).unwrap().trim().to_owned();

    //eprintln!("initial position: {}", fen);
    let board = Board::from_fen(&fen).unwrap();
    for check in splits {
        let label = check.split(" ").nth(0).unwrap().trim();
        if label.starts_with("D") {
            let depth: u32 = label
                .chars()
                .skip(1)
                .collect::<String>()
                .trim()
                .parse()
                .unwrap();
            let nodes: u64 = check.split(" ").nth(1).unwrap().trim().parse().unwrap();

            if depth > max {
                break;
            }

            let fen = fen.clone();
            let board = board.clone();
            scope.execute(move || {
                let settings = SearchSettings::divide(depth.into());
                let (count, _) = board.alphabeta(&settings, true);
                eprintln!("{} depth {} expected {} got {}", fen, depth, nodes, count);
                if false && count != nodes {
                    eprintln!("fail, here is some info:");
                    eprintln!("{}", board);
                    for m in board.legal_moves() {
                        let board = board.clone().apply_move(&m).unwrap();
                        let (ncount, _) = board.alphabeta(&settings, true);
                        eprintln!("{} count: {}", m, ncount);
                    }
                }
                assert_eq!(count, nodes);
            });
        }
    }
}
