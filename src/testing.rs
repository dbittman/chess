use serde::Deserialize;

use crate::chess::{ab::alphabeta, board::Board};

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
    pub fn start(&self) -> &TestPosition {
        &self.start
    }

    pub fn moves(&self) -> &[TestMove] {
        self.expected.as_ref()
    }
}

#[derive(Debug, Deserialize)]
pub struct TestPosition {
    description: String,
    fen: String,
}

impl TestPosition {
    pub fn fen(&self) -> &str {
        self.fen.as_ref()
    }

    pub fn description(&self) -> &str {
        self.description.as_ref()
    }
}

#[derive(Debug, Deserialize)]
pub struct TestMove {
    #[serde(alias = "move")]
    mv: String,
    fen: String,
}

impl TestMove {
    pub fn mv(&self) -> &str {
        self.mv.as_ref()
    }

    pub fn fen(&self) -> &str {
        self.fen.as_ref()
    }
}

impl TestSuite {
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    pub fn description(&self) -> &str {
        self.description.as_ref()
    }

    pub fn test_cases(&self) -> &[TestCase] {
        self.test_cases.as_ref()
    }
}

#[cfg(test)]
mod test {
    use std::{
        fs::File,
        io::{BufRead, BufReader},
        num,
        path::Path,
    };

    use crate::chess::board::Board;

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

    // #[test]
    fn test_depth() {
        test_with_epd("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1 ;D1 20 ;D2 400 ;D3 8902 ;D4 197281 ;D5 4865609 ;D6 119060324", u32::MAX);
    }

    #[test]
    fn test_depth_many_up_to_4() {
        let file = "testcases/depths.epd";
        let lines = BufReader::new(File::open(file).unwrap()).lines();

        for line in lines {
            test_with_epd(&line.unwrap(), 4);
        }
    }
}

fn test_with_epd(epd: &str, max: u32) {
    let mut splits = epd.split(";");
    let fen = splits.nth(0).unwrap().trim();

    eprintln!("initial position: {}", fen);
    let board = Board::from_fen(fen).unwrap();

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

            let (count, _) = board.alphabeta(depth.into(), true, true);
            eprintln!("depth {} expected {} got {}", depth, nodes, count);
            if count != nodes {
                eprintln!("fail, here is some info:");
                eprintln!("{}", board);
                for m in board.legal_moves() {
                    let board = board.clone().apply_move(&m).unwrap();
                    let (ncount, _) = board.alphabeta(1, true, true);
                    eprintln!("{} count: {}", m, ncount);
                }
            }
            assert_eq!(count, nodes);
        }
    }
}
