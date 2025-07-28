use crate::{
    backtest::lib::PositionState, 
    core::{candle::{Candle, CandleTrait}, signal::{Signal, SignalReason}}, 
    helper::{atr::{calculate_atr, AtrCandle}, bollinger_bands::calculate_bollinger_bands, candle::{identify_candle_pattern, CandlePattern}, ema::calculate_ema, rsi::calculate_rsi}
};

/// 캔들 패턴 전략의 상태
pub struct CandlePatternStrategyState {
    pub history_candles: Vec<Candle>,
    weight: f64, // 누적 weight (0 미만 매도 가능성 높음. 0 초과 매수 가능성 높음.)
    support_levels: Vec<f64>, // 지지선들
    resistance_levels: Vec<f64>, // 저항선들
    last_trade_candle_index: Option<usize>, // 마지막 거래 캔들 인덱스
    consecutive_losses: usize, // 연속 손실 횟수
}

/// 캔들 패턴 전략의 설정
pub struct CandlePatternStrategyConfig {
    pub enable_log: bool,
    rsi_period: usize,
    rsi_oversold: f64, // RSI 과매도 기준
    rsi_overbought: f64, // RSI 과매수 기준
    volume_threshold: f64, // 거래량 임계값 (평균 대비)
    weight_decay_rate: f64, // weight 감쇠율 (0.0 ~ 1.0)
    min_weight_for_buy: f64, // 매수 신호를 위한 최소 weight
    max_weight_for_sell: f64, // 매도 신호를 위한 최대 weight
    short_ema_period: usize, // 단기 EMA 기간
    long_ema_period: usize, // 장기 EMA 기간
    support_rsi_threshold: f64, // 지지선 계산을 위한 RSI 임계값 (낮은 값)
    resistance_rsi_threshold: f64, // 저항선 계산을 위한 RSI 임계값 (높은 값)
    stop_loss_multiplier: f64, // 손절 배수 (현재가 대비)
    take_profit_multiplier: f64, // 익절 배수 (현재가 대비)
    max_consecutive_losses: usize, // 최대 연속 손실 허용 횟수
    trend_strength_threshold: f64, // 추세 강도 임계값
    reversal_volume_multiplier: f64, // 반전 신호 거래량 배수
    ema_slope_period: usize, // EMA 기울기 계산을 위한 기간

    // --- disparity ---
    disparity_diff: f64, // 이격도 차이 임계값
}

impl CandlePatternStrategyConfig {
    pub fn new() -> Self {
        Self {
            enable_log: false,
            rsi_period: 14,
            rsi_oversold: 35.0, // 과매도 기준
            rsi_overbought: 65.0, // 과매수 기준
            volume_threshold: 1.2, // 거래량 임계값을 더 낮게
            weight_decay_rate: 0.9, // weight 감쇠율을 더 느리게
            min_weight_for_buy: 1.0, // 매수 신호 임계값을 더 낮게
            max_weight_for_sell: -1.0, // 매도 신호 임계값을 더 높게
            short_ema_period: 5,
            long_ema_period: 20,
            support_rsi_threshold: 40.0, // 지지선 RSI 임계값
            resistance_rsi_threshold: 60.0, // 저항선 RSI 임계값
            stop_loss_multiplier: 0.02, // 손절 배수 (2%)
            take_profit_multiplier: 0.04, // 익절 배수 (4%) - 손절비 1:2
            max_consecutive_losses: 5, // 최대 연속 손실 허용 횟수를 더 줄임
            trend_strength_threshold: 0.001, // 추세 강도 임계값을 더 낮게
            reversal_volume_multiplier: 1.2, // 반전 신호 거래량 배수를 더 낮게
            ema_slope_period: 3, // EMA 기울기 계산 기간

            // --- disparity ---
            disparity_diff: 0.4, // 이격도 차이 임계값  
        }
    }
}

impl CandlePatternStrategyState {
    pub fn new() -> Self {
        Self { 
            history_candles: Vec::new(), 
            weight: 0.0,
            support_levels: Vec::new(),
            resistance_levels: Vec::new(),
            last_trade_candle_index: None,
            consecutive_losses: 0,
        }
    }

    /// 새로운 캔들을 추가하고 weight를 업데이트
    pub fn add_candle(&mut self, candle: Candle, config: &CandlePatternStrategyConfig) {
        self.history_candles.push(candle);
        
        // 캔들 개수 제한 (메모리 효율성)
        if self.history_candles.len() > 100 {
            self.history_candles.remove(0);
        }
        
        // weight 감쇠 적용
        self.weight *= config.weight_decay_rate;
        
        // 지지선과 저항선 업데이트
        self.update_support_resistance_levels(config);
    }

    /// RSI 기반 지지선과 저항선 업데이트
    fn update_support_resistance_levels(&mut self, config: &CandlePatternStrategyConfig) {
        if self.history_candles.len() < config.rsi_period {
            return;
        }

        let closes = self.history_candles.iter()
            .map(|c| c.base.trade_price)
            .collect::<Vec<f64>>();
        
        let rsi_values = calculate_rsi(&closes, config.rsi_period);
        
        // 지지선 계산 (RSI가 낮은 구간의 저가들)
        let mut support_points = Vec::new();
        for (i, (candle, &rsi)) in self.history_candles.iter().zip(rsi_values.iter()).enumerate() {
            if rsi <= config.support_rsi_threshold {
                support_points.push((i as f64, candle.base.low_price));
            }
        }
        
        // 저항선 계산 (RSI가 높은 구간의 고가들)
        let mut resistance_points = Vec::new();
        for (i, (candle, &rsi)) in self.history_candles.iter().zip(rsi_values.iter()).enumerate() {
            if rsi >= config.resistance_rsi_threshold {
                resistance_points.push((i as f64, candle.base.high_price));
            }
        }
        
        // 최근 지지선과 저항선 계산 (선형 회귀 또는 평균)
        if support_points.len() >= 2 {
            let recent_support = self.calculate_trend_line(&support_points, true);
            self.support_levels.push(recent_support);
            if self.support_levels.len() > 5 {
                self.support_levels.remove(0);
            }
        }
        
        if resistance_points.len() >= 2 {
            let recent_resistance = self.calculate_trend_line(&resistance_points, false);
            self.resistance_levels.push(recent_resistance);
            if self.resistance_levels.len() > 5 {
                self.resistance_levels.remove(0);
            }
        }
    }

    /// 추세선 계산 (간단한 선형 회귀)
    fn calculate_trend_line(&self, points: &[(f64, f64)], is_support: bool) -> f64 {
        if points.len() < 2 {
            return points.first().map(|p| p.1).unwrap_or(0.0);
        }
        
        // 최근 3개 점만 사용
        let recent_points: Vec<_> = points.iter().rev().take(3).collect();
        
        // 간단한 평균 기울기 계산
        let mut slopes = Vec::new();
        for i in 0..recent_points.len() - 1 {
            let (x1, y1) = recent_points[i];
            let (x2, y2) = recent_points[i + 1];
            if x2 != x1 {
                slopes.push((y2 - y1) / (x2 - x1));
            }
        }
        
        let avg_slope = if slopes.is_empty() { 0.0 } else { slopes.iter().sum::<f64>() / slopes.len() as f64 };
        
        // 현재 시점에서의 예측값 계산
        let (last_x, last_y) = recent_points[0];
        let current_x = self.history_candles.len() as f64;
        let predicted_value = last_y + avg_slope * (current_x - last_x);
        
        // 지지선은 위로, 저항선은 아래로 조정
        if is_support {
            predicted_value * 0.995 // 지지선은 약간 아래로
        } else {
            predicted_value * 1.005 // 저항선은 약간 위로
        }
    }

    /// 매수 시 손절가 계산
    fn calculate_stop_loss(&self, current_price: f64, config: &CandlePatternStrategyConfig) -> f64 {
        // 1. 캔들 저가 기반 손절
        let last_candle = self.history_candles.last().unwrap();
        let candle_based_stop = last_candle.base.low_price * 0.99; // 저가의 1% 아래
        
        // 2. 지지선 기반 손절
        let support_based_stop = if let Some(&support) = self.support_levels.last() {
            support * 0.98 // 지지선의 2% 아래
        } else {
            current_price * (1.0 - config.stop_loss_multiplier)
        };
        
        // 3. 고정 비율 손절
        let fixed_stop = current_price * (1.0 - config.stop_loss_multiplier);
        
        // 4. ATR 기반 손절 (변동성 고려)
        let atr_based_stop = if self.history_candles.len() >= 14 {
            let atr = self.calculate_atr(14);
            current_price - (atr * 1.5) // ATR의 1.5배 아래
        } else {
            fixed_stop
        };
        
        // 가장 높은 손절가 선택 (손실 최소화)
        let calculated_stop = candle_based_stop.max(support_based_stop).max(fixed_stop).max(atr_based_stop);
        
        // 손절가가 현재가보다 높으면 안됨 (버그 방지)
        if atr_based_stop >= current_price {
            current_price * (1.0 - config.stop_loss_multiplier)
        } else {
            atr_based_stop
        }
    }

    /// 매수 시 익절가 계산
    fn calculate_take_profit(&self, current_price: f64, config: &CandlePatternStrategyConfig) -> f64 {
        // 1. 저항선 기반 익절
        let resistance_based_tp = if let Some(&resistance) = self.resistance_levels.last() {
            resistance * 0.99 // 저항선의 1% 아래
        } else {
            current_price * (1.0 + config.take_profit_multiplier)
        };
        
        // 2. 고정 비율 익절
        let fixed_tp = current_price * (1.0 + config.take_profit_multiplier);
        
        // 저항선이 있으면 저항선 기반, 없으면 고정 비율
        let calculated_tp = if self.resistance_levels.is_empty() {
            fixed_tp
        } else {
            resistance_based_tp
        };
        
        // 익절가가 현재가보다 낮으면 안됨 (버그 방지)
        if calculated_tp <= current_price {
            current_price * (1.0 + config.take_profit_multiplier)
        } else {
            calculated_tp
        }
    }

    /// 현재 추세를 파악 (EMA 기반)
    fn get_trend(&self, config: &CandlePatternStrategyConfig) -> TrendDirection {
        if self.history_candles.len() < config.long_ema_period {
            return TrendDirection::Neutral;
        }

        let closes = self.history_candles.iter()
            .map(|c| c.base.trade_price)
            .collect::<Vec<f64>>();
        
        let short_ema = calculate_ema(&closes, config.short_ema_period);
        let long_ema = calculate_ema(&closes, config.long_ema_period);
        
        let current_short_ema = *short_ema.last().unwrap_or(&0.0);
        let current_long_ema = *long_ema.last().unwrap_or(&0.0);

        let slope = self.calculate_slope(&closes, 5);
        
        // 강한 추세 조건: EMA 정렬
        // uptrend 조건: 단기 EMA > 장기 EMA
        // downtrend 조건: 단기 EMA < 장기 EMA
        let ema_aligned_up = current_short_ema > current_long_ema;
        let ema_aligned_down = current_short_ema < current_long_ema;

        let uptrend = ema_aligned_up && slope > 0.0;
        let downtrend = ema_aligned_down && slope < 0.0;
        
        // 추세 판단
        if false {
            TrendDirection::StrongUptrend
        } else if uptrend {
            TrendDirection::Uptrend
        } else if false {
            TrendDirection::StrongDowntrend
        } else if downtrend {
            TrendDirection::Downtrend
        } else if slope > 0.0 {
            TrendDirection::DowntrendNeutral
        } else if slope < 0.0 {
            TrendDirection::UptrendNeutral
        } else {
            TrendDirection::Neutral
        }
    }

    /// EMA 기울기 계산 (여러 기간의 평균 기울기)
    pub fn calculate_slope(&self, values: &[f64], period: usize) -> f64 {
        if values.len() < period + 1 {
            return 0.0;
        }
        
        let mut slopes = Vec::new();
        
        // 최근 period개 기간의 기울기들을 계산
        for i in (values.len() - period)..values.len() {
            if i > 0 {
                let slope = values[i] - values[i - 1];
                slopes.push(slope);
            }
        }
        
        // 평균 기울기 반환
        if slopes.is_empty() {
            0.0
        } else {
            slopes.iter().sum::<f64>() / slopes.len() as f64
        }
    }

    /// RSI 기반 weight 계산 (추세 추종)
    /// 양수이면 매수 신호, 음수이면 매도 신호
    fn calculate_rsi_weight(&self, config: &CandlePatternStrategyConfig) -> f64 {
        if self.history_candles.len() < config.rsi_period {
            return 0.0;
        }

        let closes = self.history_candles.iter()
            .map(|c| c.base.trade_price)
            .collect::<Vec<f64>>();
        
        let rsi_values = calculate_rsi(&closes, config.rsi_period);
        let current_rsi = rsi_values.last().unwrap_or(&50.0); // 0~100 사이의 값
        
        // 반등 전략 RSI 신호
        if *current_rsi <= config.rsi_oversold {
            // RSI가 과매도 구간이면 반등을 기대하여 매수 신호
            0.3
        } else if *current_rsi >= config.rsi_overbought {
            // RSI가 과매수 구간이면 하락을 기대하여 매도 신호
            -0.3
        } else {
            // 중립 구간에서는 신호 없음
            0.0
        }
    }

    fn calculate_disparity_weight(&self, config: &CandlePatternStrategyConfig) -> f64 {
        if self.history_candles.len() < 20 {
            return 0.0;
        }

        let current_price = self.history_candles.last().unwrap().base.trade_price;

        let closes = self.history_candles.iter()
            .map(|c| c.base.trade_price)
            .collect::<Vec<f64>>();

        let bb = calculate_bollinger_bands(&closes, 20, 1.8);
        let last_bb = bb.last().unwrap();

        let low = last_bb.lower;
        let high = last_bb.upper;
        
        // 볼린저 밴드 이격도 계산
        let band_width = high - low;
        if band_width <= 0.0 {
            return 0.0;
        }
        
        // 현재가가 밴드 내에서 어느 위치에 있는지 계산 (0.0 ~ 1.0)
        let position_in_band = (current_price - low) / band_width;
        
        // 이격도 가중치 계산
        // 하단에 닿거나 넘으면 -0.2, 상단에 닿거나 넘으면 0.2
        // 그 가운데는 smooth하게 계산
        if position_in_band <= 0.0 {
            // 하단을 넘어선 경우
            config.disparity_diff / 2.0
        } else if position_in_band >= 1.0 {
            // 상단을 넘어선 경우
            -config.disparity_diff / 2.0
        } else {
            // 밴드 내부: smooth한 선형 보간
            // 0.0 (하단) -> -0.2, 0.5 (중앙) -> 0.0, 1.0 (상단) -> 0.2
            -(position_in_band - 0.5) * config.disparity_diff
        }
    }

    /// 캔들 패턴 기반 weight 계산 (추세와 반전 신호, 거래량 결합)
    fn calculate_pattern_weight(&self, config: &CandlePatternStrategyConfig) -> f64 {
        if self.history_candles.is_empty() {
            return 0.0;
        }

        let last_candle = self.history_candles.last().unwrap();
        let pattern = identify_candle_pattern(last_candle, &self.history_candles);
        let trend = self.get_trend(config);
        
        // 현재 거래량이 평균 대비 얼마나 높은지 계산
        let (volume_ratio, avg_volume) = self.calculate_volume_ratio(config);
        
        if let Some(pattern) = &pattern {
            if config.enable_log {
                println!("캔들 패턴: {} | 추세: {:?} | 거래량 비율: {:.2}", 
                    pattern.to_korean_name(), trend, volume_ratio);
            }
        }

        // 반전 신호 패턴들 정의
        let reversal_patterns = [
            CandlePattern::ShootingStar,      // 유성형 - 상승 후 하락 반전
            CandlePattern::HangingMan,        // 교수형 - 상승 후 하락 반전
            CandlePattern::GravestoneDoji,    // 묘비형 도지 - 상승 후 하락 반전
            CandlePattern::Hammer,            // 망치형 - 하락 후 상승 반전
            CandlePattern::InvertedHammer,    // 역망치형 - 하락 후 상승 반전
            CandlePattern::DragonflyDoji,     // 잠자리형 도지 - 하락 후 상승 반전
        ];

        // 강한 추세에서 반전 신호 패턴이 나타났을 때의 로직
        if let Some(pattern) = &pattern {
            if reversal_patterns.contains(pattern) && volume_ratio > config.reversal_volume_multiplier {
                match trend {
                    TrendDirection::StrongUptrend => {
                        // 강한 상승 추세에서 하락 반전 신호 + 높은 거래량 = 매도 신호
                        match pattern {
                            CandlePattern::ShootingStar | 
                            CandlePattern::HangingMan | 
                            CandlePattern::GravestoneDoji => {
                                let strength = (volume_ratio - config.reversal_volume_multiplier) * 0.5;
                                println!("강한 상승 추세에서 하락 반전 신호 감지! 매도 weight: {:.3}", -strength);
                                return -strength;
                            }
                            _ => {}
                        }
                    }
                    TrendDirection::StrongDowntrend => {
                        // 강한 하락 추세에서 상승 반전 신호 + 높은 거래량 = 매수 신호
                        match pattern {
                            CandlePattern::Hammer | 
                            CandlePattern::InvertedHammer | 
                            CandlePattern::DragonflyDoji => {
                                let strength = (volume_ratio - config.reversal_volume_multiplier) * 0.5;
                                println!("강한 하락 추세에서 상승 반전 신호 감지! 매수 weight: {:.3}", strength);
                                return strength;
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        }

        

        // 일반적인 패턴 신호 (추세에 따라 다르게 설정)
        let base_weight = match trend {
            TrendDirection::Uptrend | TrendDirection::StrongUptrend => {
                // 상승장에서의 패턴 가중치
                match pattern {
                    // 매수 신호 패턴들 (상승 지속 또는 반등 신호)
                    Some(CandlePattern::LongBullishCandle) => 0.3, // 장대 양봉형
                    Some(CandlePattern::Hammer) => 0.1, // 망치형
                    Some(CandlePattern::BottomTailBullish) => 0.1, // 밑꼬리양봉형
                    Some(CandlePattern::DragonflyDoji) => 0.1, // 잠자리형
                    Some(CandlePattern::RisingShootingStar) => 0.1, // 상승 샅바형

                    Some(CandlePattern::InvertedHammer) => -0.1, // 역망치형
                    Some(CandlePattern::LongBearishCandle) => -0.3, // 장대 음봉형
                    Some(CandlePattern::TopTailBearish) => -0.1, // 윗꼬리음봉형
                    Some(CandlePattern::ShootingStar) => -0.1, // 유성형
                    Some(CandlePattern::GravestoneDoji) => -0.2, // 비석 십자형
                    Some(CandlePattern::HangingMan) => -0.1, // 교수형
                    Some(CandlePattern::FallingShootingStar) => -0.2, // 하락 샅바형
                    
                    // 중립 패턴들
                    Some(CandlePattern::Doji) => 0.0, // 십자형
                    Some(CandlePattern::FourPriceDoji) => 0.0, // 점십자형
                    None => 0.1, // 패턴 없음
                }
            },
            TrendDirection::Downtrend | TrendDirection::StrongDowntrend => {
                // 하락장에서의 패턴 가중치
                match pattern {
                    // 매수 신호 패턴들 (하락장에서 반등 신호)
                    Some(CandlePattern::Hammer) => 0.1, // 망치형
                    Some(CandlePattern::InvertedHammer) => 0.1, // 역망치형
                    Some(CandlePattern::BottomTailBullish) => 0.1, // 밑꼬리양봉형
                    Some(CandlePattern::DragonflyDoji) => 0.1, // 잠자리형
                    Some(CandlePattern::RisingShootingStar) => 0.1, // 상승 샅바형
                    Some(CandlePattern::Doji) => 0.1, // 십자형
                    Some(CandlePattern::LongBullishCandle) => 0.3, // 장대 양봉형 

                    Some(CandlePattern::LongBearishCandle) => -0.3, // 장대 음봉형
                    Some(CandlePattern::TopTailBearish) => -0.1, // 윗꼬리음봉형
                    Some(CandlePattern::ShootingStar) => -0.1, // 유성형
                    Some(CandlePattern::GravestoneDoji) => 0.2, // 비석 십자형
                    Some(CandlePattern::HangingMan) => -0.1, // 교수형
                    Some(CandlePattern::FallingShootingStar) => -0.1, // 하락 샅바형
                    
                    // 중립 패턴들
                    Some(CandlePattern::FourPriceDoji) => 0.0, // 점십자형
                    None => -0.1, // 패턴 없음
                }
            },
            TrendDirection::Neutral => {
                // 중립장에서의 패턴 가중치 (보수적으로 설정)
                match pattern {
                    // 강한 신호들
                    Some(CandlePattern::LongBullishCandle) => 0.3, // 장대 양봉형
                    Some(CandlePattern::LongBearishCandle) => -0.3, // 장대 음봉형
                    Some(CandlePattern::Hammer) => 0.1, // 망치형
                    Some(CandlePattern::BottomTailBullish) => 0.1, // 상승샅바형
                    // 중립 패턴들
                    _ => 0.0,
                }
            },
            // 상승 추세 중 중립(횡보) 패턴
            TrendDirection::UptrendNeutral => {
                match pattern {
                    // 상승 추세 지속을 암시하는 강한 매수 신호
                    Some(CandlePattern::LongBullishCandle) => 0.3, // 장대 양봉형
                    Some(CandlePattern::Hammer) => 0.2, // 망치형
                    Some(CandlePattern::BottomTailBullish) => 0.2, // 밑꼬리양봉형

                    // 추세 반전을 경고하는 강한 매도 신호
                    Some(CandlePattern::LongBearishCandle) => -0.3, // 장대 음봉형
                    Some(CandlePattern::ShootingStar) => -0.2, // 유성형
                    Some(CandlePattern::HangingMan) => -0.2, // 교수형
                    Some(CandlePattern::GravestoneDoji) => -0.2, // 비석 십자형

                    // 횡보 상태(불확실성)를 나타내는 중립 신호
                    Some(CandlePattern::Doji) => 0.0, // 십자형
                    Some(CandlePattern::InvertedHammer) => 0.0, // 역망치형 (상승 추세 중에는 신뢰도 하락)
                    None => 0.0, // 특정 패턴 없음
                    _ => 0.0,
                }
            },
            // 하락 추세 중 중립(횡보) 패턴
            TrendDirection::DowntrendNeutral => {
                match pattern {
                    // 하락 추세 반전을 암시하는 강한 매수 신호
                    Some(CandlePattern::LongBullishCandle) => 0.3, // 장대 양봉형
                    Some(CandlePattern::Hammer) => 0.2, // 망치형
                    Some(CandlePattern::InvertedHammer) => 0.2, // 역망치형 (하락 추세 중에는 반등 신호로 해석)
                    Some(CandlePattern::DragonflyDoji) => 0.2, // 잠자리형

                    // 하락 추세 지속을 암시하는 강한 매도 신호
                    Some(CandlePattern::LongBearishCandle) => -0.3, // 장대 음봉형
                    Some(CandlePattern::ShootingStar) => -0.2, // 유성형
                    Some(CandlePattern::FallingShootingStar) => -0.2, // 하락 샅바형

                    // 횡보 상태(불확실성)를 나타내는 중립 신호
                    Some(CandlePattern::Doji) => 0.0, // 십자형
                    Some(CandlePattern::HangingMan) => 0.0, // 교수형 (하락 추세 중에는 신뢰도 하락)
                    None => 0.0, // 특정 패턴 없음
                    _ => 0.0,
                }
            },
        };
        
        // --- 거래량, 몸통, 꼬리 신뢰도 가중치 계산 ---
        let body_size = (last_candle.base.trade_price - last_candle.base.opening_price).abs();
        let upper_shadow_size = last_candle.base.high_price - last_candle.base.trade_price.max(last_candle.base.opening_price);
        let lower_shadow_size = last_candle.base.opening_price.min(last_candle.base.trade_price) - last_candle.base.low_price;
        let total_range = last_candle.base.high_price - last_candle.base.low_price;

        // 거래량 가중치 
        let volume_weight = if volume_ratio > 1.2 {
            1.0 + (volume_ratio - 1.0) * 0.3
        } else if volume_ratio < 0.8 {
            0.8
        } else {
            1.0
        };

        // 몸통 가중치 (장대양봉/음봉, 샅바형 등)
        let body_ratio = if total_range > 0.0 { body_size / total_range } else { 0.0 };
        let body_weight = match pattern {
            Some(CandlePattern::LongBullishCandle) | Some(CandlePattern::LongBearishCandle) => {
                if body_ratio > 0.8 { 0.15 } else if body_ratio > 0.7 { 0.1 } else { 0.0 }
            },
            Some(CandlePattern::RisingShootingStar) | Some(CandlePattern::FallingShootingStar) => {
                if body_ratio > 0.5 { 0.1 } else { 0.0 }
            },
            _ => 0.0
        };

        // 꼬리 가중치 (패턴별로 다르게)
        let lower_shadow_ratio = if total_range > 0.0 { lower_shadow_size / total_range } else { 0.0 };
        let upper_shadow_ratio = if total_range > 0.0 { upper_shadow_size / total_range } else { 0.0 };
        let shadow_weight = match pattern {
            Some(CandlePattern::Hammer) | Some(CandlePattern::BottomTailBullish) => {
                if lower_shadow_ratio > 0.6 { 0.1 } else { 0.0 }
            },
            Some(CandlePattern::ShootingStar) | Some(CandlePattern::TopTailBearish) | Some(CandlePattern::GravestoneDoji) => {
                if upper_shadow_ratio > 0.6 { 0.1 } else { 0.0 }
            },
            Some(CandlePattern::DragonflyDoji) => {
                if lower_shadow_ratio > 0.5 { 0.1 } else { 0.0 }
            },
            _ => 0.0
        };

        // 최종 가중치 계산
        base_weight * volume_weight + body_weight + shadow_weight
    }

    /// 거래량 비율 계산
    fn calculate_volume_ratio(&self, config: &CandlePatternStrategyConfig) -> (f64, f64) {
        if self.history_candles.len() < 20 {
            return (1.0, 0.0);
        }

        let last_candle = self.history_candles.last().unwrap();
        let current_volume = last_candle.base.candle_acc_trade_volume;
        
        // 최근 20개 캔들의 평균 거래량 계산
        let avg_volume = self.history_candles.iter()
            .rev()
            .take(20)
            .map(|c| c.base.candle_acc_trade_volume)
            .sum::<f64>() / 20.0;
        
        (current_volume / avg_volume, avg_volume)
    }

    /// 연속 손실 횟수 업데이트
    pub fn update_consecutive_losses(&mut self, is_loss: bool) {
        if is_loss {
            self.consecutive_losses += 1;
        } else {
            self.consecutive_losses = 0; // 승리 시 리셋
        }
    }

    /// ATR (Average True Range) 계산
    fn calculate_atr(&self, period: usize) -> f64 {
        if self.history_candles.len() < period + 1 {
            return 0.0;
        }

        let mut true_ranges = Vec::new();
        
        for i in 1..self.history_candles.len() {
            let current = &self.history_candles[i];
            let previous = &self.history_candles[i - 1];
            
            let high_low = current.base.high_price - current.base.low_price;
            let high_close = (current.base.high_price - previous.base.trade_price).abs();
            let low_close = (current.base.low_price - previous.base.trade_price).abs();
            
            let true_range = high_low.max(high_close).max(low_close);
            true_ranges.push(true_range);
        }
        
        // 최근 period개의 평균
        true_ranges.iter().rev().take(period).sum::<f64>() / period as f64
    }

    /// 거래량 기반 weight 계산 (기존 방식 유지하되 가중치 낮춤)
    fn calculate_volume_weight(&self, config: &CandlePatternStrategyConfig) -> f64 {
        if self.history_candles.len() < 20 {
            return 0.0;
        }

        let (volume_ratio, avg_volume) = self.calculate_volume_ratio(config);

        volume_ratio
    }
}

/// 추세 방향을 나타내는 열거형
#[derive(Debug, Clone, PartialEq)]
enum TrendDirection {
    StrongUptrend,   // 강한 상승 추세
    Uptrend,         // 상승 추세
    Neutral,
    Downtrend,       // 하락 추세
    StrongDowntrend, // 강한 하락 추세
    UptrendNeutral, // 상승 추세 중립
    DowntrendNeutral, // 하락 추세 중립
}

pub fn candle_pattern_strategy(
    state: &mut CandlePatternStrategyState, 
    config: &CandlePatternStrategyConfig, 
    position: &mut PositionState,
    new_candle: Option<Candle>
) -> Signal {
    // 새로운 캔들이 있으면 추가
    if let Some(candle) = new_candle {
        state.add_candle(candle, config);
    }
    
    // 최소 캔들 개수 확인
    if state.history_candles.len() < config.long_ema_period {
        return Signal::Hold;
    }
    
    // 각 요소별 weight 계산
    let rsi_weight = state.calculate_rsi_weight(config);
    let pattern_weight = state.calculate_pattern_weight(config);
    let volume_weight = state.calculate_volume_weight(config);
    let disparity_weight = state.calculate_disparity_weight(config);
    
    // 현재 weight 계산
    let current_weight = rsi_weight + pattern_weight + disparity_weight * volume_weight;
    
    // 누적 weight 업데이트 (decay factor 적용)
    state.weight = state.weight * config.weight_decay_rate + current_weight;

    // 누적 weight 제한
    state.weight = state.weight.clamp(-3.0, 3.0);

    // 추세 정보 출력
    let trend = state.get_trend(config);
    
    // 최소 거래 간격 확인 (최소 3개 캔들 간격으로 늘림)
    let min_trade_interval = 3;
    let current_candle_index = state.history_candles.len() - 1;


    // if let Some(last_trade_index) = state.last_trade_candle_index {
    //     if current_candle_index - last_trade_index < min_trade_interval {
    //         return Signal::Hold;
    //     }
    // }

    if config.enable_log {
        println!("가격: {} | 추세: {:?} | rsi_weight: {:.3} | pattern_weight: {:.3} | volume_weight: {:.3} | disparity_weight: {:.3} | 누적_weight: {:.3}"
        , state.history_candles.last().unwrap().base.trade_price, trend, rsi_weight, pattern_weight, volume_weight, disparity_weight, state.weight);
    }
    
    // 기본 임계값 사용
    let buy_threshold = config.min_weight_for_buy;
    let sell_threshold = config.max_weight_for_sell;

    // 연속 손실이 너무 많으면 전략 일시 중단
    // if state.consecutive_losses >= config.max_consecutive_losses {
    //     return Signal::Hold;
    // }

    // 포지션 상태에 따른 신호 결정
    match position {
        PositionState::None => {
            // 포지션이 없을 때 - 매수 신호 확인
            if (state.weight >= buy_threshold * 2.0 && trend == TrendDirection::Downtrend) || (state.weight >= buy_threshold && trend != TrendDirection::Downtrend) {
                // // 단순한 모멘텀 전략: 추세 필터 제거
                // // RSI 필터: 과매도 구간에서만 매수
                // let rsi_weight = state.calculate_rsi_weight(config);
                // if rsi_weight < 0.0 {
                //     return Signal::Hold;
                // }
                
                let current_price = state.history_candles.last().unwrap().base.trade_price;
                let stop_loss = state.calculate_stop_loss(current_price, config);
                let take_profit = state.calculate_take_profit(current_price, config);
                
                // 손절/익절 가격 검증
                if stop_loss >= current_price {
                    println!("경고: 손절가({:.0})가 현재가({:.0})보다 높음. 고정 비율로 조정", stop_loss, current_price);
                }
                if take_profit <= current_price {
                    println!("경고: 익절가({:.0})가 현재가({:.0})보다 낮음. 고정 비율로 조정", take_profit, current_price);
                }
                
                // 거래 인덱스 업데이트
                state.last_trade_candle_index = Some(current_candle_index);
                
                Signal::Buy {
                    reason: format!("캔들 패턴 전략 - 누적 Weight: {:.3}, 현재 Weight: {:.3}, 손절: {:.0}, 익절: {:.0}", 
                        state.weight, current_weight, stop_loss, take_profit),
                    initial_trailing_stop: stop_loss,
                    take_profit: current_price + (current_price - stop_loss) * 4.0,
                    asset_pct: 1.0, // 전체 자산 사용
                }
            } else {
                Signal::Hold
            }
        }
        PositionState::InPosition { entry_price: _, entry_asset: _, take_profit_price: _, trailing_stop_price: _ } => {
            // 포지션이 있을 때 - 매도 신호 확인
            if state.weight <= sell_threshold {
                // 거래 인덱스 업데이트
                state.last_trade_candle_index = Some(current_candle_index);
                
                Signal::Sell(SignalReason {
                    reason: format!("캔들 패턴 전략 - 누적 Weight: {:.3}, 현재 Weight: {:.3}", state.weight, current_weight),
                })
            } else {
                Signal::Hold
            }
        }
    }
}
