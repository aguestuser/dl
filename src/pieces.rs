pub trait PieceMaker {
    fn calc_piece_size(&self, file_size: u64) -> u64;
}

pub enum PieceMakerImpl {
    Static(StaticPieceMaker),
}

pub struct StaticPieceMaker {
    piece_size: u64,
}

impl StaticPieceMaker {
    pub fn new(piece_size: u64) -> StaticPieceMaker {
        Self { piece_size }
    }
}

impl PieceMaker for StaticPieceMaker {
    fn calc_piece_size(&self, _file_size: u64) -> u64 {
        self.piece_size
    }
}

pub struct DynamicPieceMaker {
    divisor: u64,
}

impl DynamicPieceMaker {
    pub fn new(divisor: u64) -> DynamicPieceMaker {
        Self { divisor }
    }
}

impl PieceMaker for DynamicPieceMaker {
    fn calc_piece_size(&self, file_size: u64) -> u64 {
        file_size / &self.divisor
    }
}

#[cfg(test)]
mod pieces_test {
    use super::*;

    #[test]
    fn constructing_static_piece_maker() {
        assert_eq!(StaticPieceMaker::new(1).piece_size, 1);
    }

    #[test]
    fn calculating_a_static_piece_size() {
        assert_eq!(StaticPieceMaker::new(1).calc_piece_size(100), 1);
    }

    #[test]
    fn constructing_dynamic_piece_maker() {
        assert_eq!(DynamicPieceMaker::new(2).divisor, 2);
    }

    #[test]
    fn calculating_a_dynamic_piece_size() {
        assert_eq!(DynamicPieceMaker::new(2).calc_piece_size(200), 100);
    }

}
