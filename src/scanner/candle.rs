use std::env;

use crate::error::{Result, RsAlgoError, RsAlgoErrorKind};
use crate::helpers::comp::percentage_change;
use crate::helpers::date::*;
use serde::{Deserialize, Serialize};

pub type OHLCV = (f64, f64, f64, f64);
pub type DOHLCV = (DateTime<Local>, f64, f64, f64, f64, f64);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CandleType {
    Default,
    Doji,
    Karakasa,
    BearishKarakasa,
    Marubozu,
    BearishMarubozu,
    Harami,
    BearishHarami,
    BearishStar,
    Engulfing,
    MorningStar,
    BearishEngulfing,
    HangingMan,
    BullishCrows,
    BearishCrows,
    BullishGap,
    BearishGap,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Candle {
    pub candle_type: CandleType,
    pub date: DateTime<Local>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub is_closed: bool,
}

impl Candle {
    pub fn new() -> CandleBuilder {
        CandleBuilder::new()
    }
    pub fn date(&self) -> DateTime<Local> {
        self.date
    }

    pub fn set_date(&mut self, date: DateTime<Local>) -> DateTime<Local> {
        self.date = date;
        self.date
    }

    pub fn open(&self) -> f64 {
        self.open
    }

    pub fn high(&self) -> f64 {
        self.high
    }

    pub fn set_high(&mut self, value: f64) -> f64 {
        self.high = value;
        self.high
    }

    pub fn low(&self) -> f64 {
        self.low
    }

    pub fn set_low(&mut self, value: f64) -> f64 {
        self.low = value;
        self.low
    }

    pub fn set_is_closed(&mut self, val: bool) -> bool {
        self.is_closed = val;
        self.is_closed
    }

    pub fn close(&self) -> f64 {
        self.close
    }

    pub fn set_close(&mut self, value: f64) -> f64 {
        self.close = value;
        self.close
    }

    pub fn set_open(&mut self, value: f64) -> f64 {
        self.open = value;
        self.open
    }

    pub fn is_closed(&self) -> bool {
        self.is_closed
    }

    pub fn volume(&self) -> f64 {
        self.volume
    }

    pub fn candle_type(&self) -> &CandleType {
        &self.candle_type
    }

    pub fn is_bullish(&self) -> bool {
        self.candle_type == CandleType::Engulfing
            || self.candle_type == CandleType::Karakasa
            || self.candle_type == CandleType::MorningStar
    }

    pub fn is_bearish(&self) -> bool {
        self.candle_type == CandleType::BearishEngulfing
            || self.candle_type == CandleType::BearishKarakasa
            || self.candle_type == CandleType::BearishStar
    }

    pub fn from_logarithmic_values(&self) -> Self {
        Self {
            date: self.date,
            open: self.open.exp(),
            high: self.high.exp(),
            low: self.low.exp(),
            close: self.close.exp(),
            volume: self.volume,
            is_closed: self.is_closed(),
            candle_type: self.candle_type.clone(),
        }
    }
}

pub struct CandleBuilder {
    date: Option<DateTime<Local>>,
    open: Option<f64>,
    high: Option<f64>,
    low: Option<f64>,
    close: Option<f64>,
    volume: Option<f64>,
    is_closed: Option<bool>,
    previous_candles: Option<Vec<DOHLCV>>,
    logarithmic: Option<bool>,
}

impl CandleBuilder {
    pub fn new() -> Self {
        Self {
            date: None,
            open: None,
            high: None,
            low: None,
            close: None,
            volume: None,
            is_closed: None,
            previous_candles: None,
            logarithmic: None,
        }
    }

    pub fn date(mut self, val: DateTime<Local>) -> Self {
        self.date = Some(val);
        self
    }

    pub fn open(mut self, val: f64) -> Self {
        self.open = Some(val);
        self
    }

    pub fn high(mut self, val: f64) -> Self {
        self.high = Some(val);
        self
    }

    pub fn low(mut self, val: f64) -> Self {
        self.low = Some(val);
        self
    }

    pub fn close(mut self, val: f64) -> Self {
        self.close = Some(val);
        self
    }

    pub fn volume(mut self, val: f64) -> Self {
        self.volume = Some(val);
        self
    }

    pub fn is_closed(mut self, val: bool) -> Self {
        self.is_closed = Some(val);
        self
    }

    pub fn previous_candles(mut self, val: Vec<DOHLCV>) -> Self {
        self.previous_candles = Some(val);
        self
    }

    pub fn logarithmic(mut self, val: bool) -> Self {
        self.logarithmic = Some(val);
        self
    }

    fn get_current_ohlc(&self) -> OHLCV {
        match self.logarithmic.unwrap() {
            true => (
                self.open.unwrap().exp(),
                self.high.unwrap().exp(),
                self.low.unwrap().exp(),
                self.close.unwrap().exp(),
            ),
            false => (
                self.open.unwrap(),
                self.high.unwrap(),
                self.low.unwrap(),
                self.close.unwrap(),
            ),
        }
    }

    fn get_previous_ohlc(&self, index: usize) -> OHLCV {
        match self.logarithmic.unwrap() {
            true => (
                self.previous_candles.as_ref().unwrap()[index].1,
                self.previous_candles.as_ref().unwrap()[index].2,
                self.previous_candles.as_ref().unwrap()[index].3,
                self.previous_candles.as_ref().unwrap()[index].4,
            ),
            false => (
                self.previous_candles.as_ref().unwrap()[index].1.exp(),
                self.previous_candles.as_ref().unwrap()[index].2.exp(),
                self.previous_candles.as_ref().unwrap()[index].3.exp(),
                self.previous_candles.as_ref().unwrap()[index].4.exp(),
            ),
        }
    }

    fn is_doji(&self) -> bool {
        // (O = C ) || (ABS(O – C ) <= ((H – L ) * 0.1))
        let (open, high, low, close) = &self.get_current_ohlc();
        (open.floor() == close.floor()) || (open - close).abs() <= ((high - low) * 0.1)
    }

    fn is_karakasa(&self) -> bool {
        // ((H-L)>3*(O-C)AND((C-L)/(.001+H-L)>0.6)AND((O-L)/(.001+H-L)>0.6))
        let (open, high, low, close) = &self.get_current_ohlc();
        (high - low) > 3. * (open - close)
            && ((close - low) / (0.001 + high - low) > 0.6)
            && ((open - low) / (0.001 + high - low) > 0.6)
    }

    fn is_bearish_karakasa(&self) -> bool {
        // (((H – L) > 3 * (O – C)) AND ((H – C) / (.001 + H – L) > 0.6) AND ((H – O) / (.001 + H – L) > 0.6))
        let (open, high, low, close) = &self.get_current_ohlc();
        ((high - low) > 3. * (open - close))
            && ((high - close) / (0.001 + high - low) > 0.6)
            && ((high - open) / (0.001 + high - low) > 0.6)
    }

    fn is_bullish_star(&self) -> bool {
        // ((O2>C2)AND((O2-C2)/(.001+H2-L2)>.6)AND(C2>O1) AND(O1>C1)AND((H1-L1)>(3*(C1-O1))) AND(C>O)AND(O>O1))
        let (open, _high, _low, close) = &self.get_current_ohlc();
        let (prev_open, prev_high, prev_low, prev_close) = &self.get_previous_ohlc(0);
        let (prev_open1, prev_high1, prev_low1, prev_close1) = &self.get_previous_ohlc(1);
        (prev_open1 > prev_close1)
            && ((prev_open1 - prev_close1) / (0.001 + prev_high1 - prev_low1) > 0.6)
            && (prev_close1 > prev_open)
            && (prev_open > prev_close)
            && ((prev_high - prev_low) > (3. * (prev_close - prev_open)))
            && (close > open)
            && (open > prev_open)
    }

    fn is_bearish_star(&self) -> bool {
        // ((O2>C2)AND((O2-C2)/(.001+H2-L2)>.6)AND(C2>O1) AND(O1>C1)AND((H1-L1)>(3*(C1-O1))) AND(C>O)AND(O>O1))
        let (open, _high, _low, close) = &self.get_current_ohlc();
        let (prev_open, prev_high, prev_low, prev_close) = &self.get_previous_ohlc(0);
        let (prev_open1, prev_high1, prev_low1, prev_close1) = &self.get_previous_ohlc(1);
        (prev_open1 > prev_close1)
            && ((prev_open1 - prev_close1) / (0.001 + prev_high1 - prev_low1) > 0.6)
            && (prev_close1 < prev_open)
            && (prev_open > prev_close)
            && ((prev_high - prev_low) > (3. * (prev_close - prev_open)))
            && (close > open)
            && (open < prev_open)
    }

    fn is_marubozu(&self) -> bool {
        //O = L AND H = C.
        let (open, high, low, close) = &self.get_current_ohlc();
        let high_shadow = (high - close) / close;
        let low_shadow = (low - open) / open;
        (open <= low && low_shadow < 0.1) && (high >= close && high_shadow < 0.1)
    }

    fn is_bearish_marubozu(&self) -> bool {
        //O = H AND C = L.
        let (open, high, low, close) = &self.get_current_ohlc();
        let high_shadow = (high - open) / open;
        let _low_shadow = (low - close) / close;
        (open >= high && high_shadow < 0.1) && (low <= close && high_shadow < 0.1)
    }

    fn is_hanging_man(&self) -> bool {
        // (((H – L) > 4 * (O – C)) AND ((C – L) / (.001 + H – L) >= 0.75) AND ((O – L) / (.001 + H – L) >= .075)))
        let (open, high, low, close) = &self.get_current_ohlc();
        ((high - low) > 4. * (open - close))
            && ((close - low) / (0.001 + high - low) > 0.75)
            && ((open - low) / (0.001 + high - low) > 0.75)
    }

    fn is_engulfing(&self) -> bool {
        //(O1 > C1) AND (C > O) AND (C >= O1) AND (C1 >= O) AND ((C – O) > (O1 – C1))
        let (open, _high, _low, close) = &self.get_current_ohlc();
        let (prev_open, _prev_high, _prev_low, prev_close) = &self.get_previous_ohlc(0);
        (prev_open > prev_close)
            && (close > open)
            && (close >= prev_open)
            && (prev_close >= open)
            && ((close - open) > (prev_open - prev_close))
    }

    fn is_bearish_engulfing(&self) -> bool {
        //(C1 > O1) AND (O > C) AND (O >= C1) AND (O1 >= C) AND ((O – C) > (C1 – O1))
        let (open, _high, _low, close) = &self.get_current_ohlc();
        let (prev_open, _prev_high, _prev_low, prev_close) = &self.get_previous_ohlc(0);
        (prev_close > prev_open)
            && (open > close)
            && (open >= prev_close)
            && (prev_open >= close)
            && ((open - close) > (prev_close - prev_open))
    }

    fn is_harami(&self) -> bool {
        //((O1 > C1) AND (C > O) AND (C <= O1) AND (C1 <= O) AND ((C – O) < (O1 – C1)))
        let (open, _high, _low, close) = &self.get_current_ohlc();
        let (prev_open, _prev_high, _prev_low, prev_close) = &self.get_previous_ohlc(0);
        (prev_open > prev_close)
            && (close > open)
            && (close <= prev_open)
            && (prev_close <= open)
            && ((close - open) < (prev_open - prev_close))
    }

    fn is_bearish_harami(&self) -> bool {
        //((C1 > O1) AND (O > C) AND (O <= C1) AND (O1 <= C) AND ((O – C) < (C1 – O1)))
        let (open, _high, _low, close) = &self.get_current_ohlc();
        let (prev_open, _prev_high, _prev_low, prev_close) = &self.get_previous_ohlc(0);
        (prev_close > prev_open)
            && (open > close)
            && (open <= prev_close)
            && (prev_open <= close)
            && ((open - close) < (prev_close - prev_open))
    }

    fn is_bullish_gap(&self) -> bool {
        //((C1 > O1) AND (O > C) AND (O <= C1) AND (O1 <= C) AND ((O – C) < (C1 – O1)))
        let (open, _high, _low, close) = &self.get_current_ohlc();
        let (_prev_open, prev_high, _prev_low, _prev_close) = &self.get_previous_ohlc(0);
        let percentage_diff = percentage_change(*prev_high, *open);
        open > prev_high && percentage_diff > 3. && close > prev_high
    }

    fn is_bearish_gap(&self) -> bool {
        //FIXME
        //((C1 > O1) AND (O > C) AND (O <= C1) AND (O1 <= C) AND ((O – C) < (C1 – O1)))
        let (open, _high, _low, close) = &self.get_current_ohlc();
        let (_a, _prev_high, prev_low, _prev_close) = &self.get_previous_ohlc(0);
        let percentage_diff = percentage_change(*prev_low, *open);
        open < prev_low && percentage_diff > 3. && close < prev_low
    }

    fn is_bullish_crows(&self) -> bool {
        //(C>O*1.01) AND(C1>O1*1.01) AND(C2>O2*1.01) AND(C>C1) AND
        // (C1>C2) AND(OO1) AND(O1O2) AND (((H-C)/(H-L))<.2) AND(((H1-C1)/(H1-L1))<.2)AND(((H2-C2)/(H2-L2))<.2)
        let (open, high, low, close) = &self.get_current_ohlc();
        let (prev_open, prev_high, prev_low, prev_close) = &self.get_previous_ohlc(0);
        let (prev_open1, prev_high1, prev_low1, prev_close1) = &self.get_previous_ohlc(1);

        (close > &(open * 1.01))
            && (prev_close > &(prev_open * 1.01))
            && (prev_close1 > &(prev_open1 * 1.01))
            && (close > prev_close)
            && (prev_close > prev_close1)
            && (((high - close) / (high - low) < 0.2)
                && ((prev_high - prev_close) / (prev_high - prev_low) < 0.2)
                && ((prev_high1 - prev_close1) / (prev_high1 - prev_low1) < 0.2))
    }

    fn is_bearish_crows(&self) -> bool {
        //(C>O*1.01) AND(C1>O1*1.01) AND(C2>O2*1.01) AND(C>C1) AND
        // (C1>C2) AND(OO1) AND(O1O2) AND (((H-C)/(H-L))<.2) AND(((H1-C1)/(H1-L1))<.2)AND(((H2-C2)/(H2-L2))<.2)
        let (open, high, low, close) = &self.get_current_ohlc();
        let (prev_open, prev_high, prev_low, prev_close) = &self.get_previous_ohlc(0);
        let (prev_open1, prev_high1, prev_low1, prev_close1) = &self.get_previous_ohlc(1);

        (open > &(close * 1.01))
            && (prev_open > &(prev_close * 1.01))
            && (prev_open1 > &(prev_close1 * 1.01))
            && (close < prev_close)
            && (prev_close < prev_close1)
            && (open > prev_close)
            && (open < prev_open)
            && (prev_open > prev_close1)
            && (prev_open < prev_open1)
            && (((close - low) / (high - low) < 0.2)
                && ((prev_close - prev_low) / (prev_high - prev_low) < 0.2)
                && ((prev_close1 - prev_low1) / (prev_high1 - prev_low1) < 0.2))
    }

    fn identify_candle_type(&self) -> CandleType {
        let candle_types = env::var("CANDLE_TYPES").unwrap().parse::<bool>().unwrap();

        match candle_types {
            true => {
                if self.is_bullish_gap() {
                    CandleType::BullishGap
                } else if self.is_karakasa() {
                    CandleType::Karakasa
                } else if self.is_bullish_star() {
                    CandleType::MorningStar
                } else if self.is_bullish_crows() {
                    CandleType::BullishCrows
                } else if self.is_marubozu() {
                    CandleType::Marubozu
                } else if self.is_engulfing() {
                    CandleType::Engulfing
                } else if self.is_bearish_karakasa() {
                    CandleType::BearishKarakasa
                } else if self.is_bearish_star() {
                    CandleType::BearishStar
                } else if self.is_hanging_man() {
                    CandleType::HangingMan
                } else if self.is_bearish_gap() {
                    CandleType::BearishGap
                } else if self.is_bearish_crows() {
                    CandleType::BearishCrows
                } else if self.is_bearish_marubozu() {
                    CandleType::BearishMarubozu
                } else if self.is_bearish_engulfing() {
                    CandleType::BearishEngulfing
                } else if self.is_harami() {
                    CandleType::Harami
                } else if self.is_bearish_harami() {
                    CandleType::BearishHarami
                } else if self.is_doji() {
                    CandleType::Doji
                } else {
                    CandleType::Default
                }
            }
            false => CandleType::Default,
        }
    }

    pub fn build(self) -> Result<Candle> {
        if let (
            Some(date),
            Some(open),
            Some(high),
            Some(low),
            Some(close),
            Some(volume),
            Some(is_closed),
            Some(_previous_candles),
            Some(_logarithmic),
        ) = (
            self.date,
            self.open,
            self.high,
            self.low,
            self.close,
            self.volume,
            self.is_closed,
            self.previous_candles.as_ref(),
            self.logarithmic,
        ) {
            Ok(Candle {
                candle_type: self.identify_candle_type(),
                date,
                open,
                close,
                high,
                low,
                volume,
                is_closed,
            })
        } else {
            Err(RsAlgoError {
                err: RsAlgoErrorKind::InvalidCandle,
            })
        }
    }
}
