use crate::core::signal::Signal;
use std::ops::Add;

#[derive(Clone, Copy, Debug)]
pub struct BacktestParams {
    pub fees_pct: f64,
    pub take_profit_pct: f64,
}

// 가상 포지션의 상태
#[derive(Debug, Clone, Copy)]
pub enum PositionState {
    None, // 포지션 없음
    InPosition {
        entry_price: f64,
        entry_asset: f64,
        take_profit_price: f64,
        trailing_stop_price: f64, // 트레일링 스탑 가격
    },
}

const INITIAL_ASSET: f64 = 1000000.0;

#[derive(Clone, Debug)]
// 백테스터의 전체 상태와 결과를 관리
pub struct BacktesterState {
    pub position: PositionState,
    pub params: BacktestParams,
    // --- 성과 측정 지표 ---
    pub win_count: u32,
    pub loss_count: u32,
    pub total_pnl_pct: f64, // 총 누적 손익률
    
    // -- 현재 자산 --
    pub current_asset: f64,
}

impl BacktesterState {
    pub fn new(params: BacktestParams) -> Self {
        BacktesterState {
            position: PositionState::None,
            params,
            win_count: 0,
            loss_count: 0,
            total_pnl_pct: 0.0,
            current_asset: INITIAL_ASSET,
        }
    }

    pub fn get_position(&mut self) -> &mut PositionState {
        &mut self.position
    }
    
    /// 매 프레임마다 현재 가격을 체크하여 포지션을 청산할 지 결정
    pub fn check_and_close_position(&mut self, current_price: f64) {
        if let PositionState::InPosition { entry_price, entry_asset, take_profit_price, trailing_stop_price } = self.position {
            let mut pnl_pct = 0.0; // 손익률
            let mut closed = false;

            // 익절 조건
            if current_price >= take_profit_price {
                pnl_pct = (take_profit_price / entry_price - 1.0) - self.params.fees_pct * 2.0;
                closed = true;
            }
            
            // 트레일링 스탑
            else if current_price <= trailing_stop_price {
                let exit_price = trailing_stop_price;
                pnl_pct = (exit_price / entry_price - 1.0) - self.params.fees_pct * 2.0;
                closed = true;
            }

            if closed {
                self.current_asset += entry_asset * (1.0 + pnl_pct);
                self.total_pnl_pct = (self.current_asset / INITIAL_ASSET) - 1.0;
                self.position = PositionState::None; // 포지션 청산
                if pnl_pct > 0.0 {
                    self.win_count += 1;
                    println!("\x1b[32m[익절] 진입가: {:.0}, 목표가: {:.0}, 현재가: {:.0}, 수익률: {:.4}%\x1b[0m", 
                            entry_price, take_profit_price, current_price, pnl_pct * 100.0);
                } else {
                    self.loss_count += 1;
                    println!("\x1b[31m[손절] 진입가: {:.0}, 트레일링스탑: {:.0}, 현재가: {:.0}, 손실률: {:.4}%\x1b[0m", 
                            entry_price, trailing_stop_price, current_price, pnl_pct * 100.0);
                }
                self.print_results(); // 중간 결과 출력
            } else {
                // 포지션 상태 갱신 (트레일링 스탑 가격 반영)
                self.position = PositionState::InPosition {
                    entry_price,
                    entry_asset,
                    take_profit_price,
                    trailing_stop_price,
                };
            }
        }
    }

    /// 전략 신호에 따라 포지션을 관리 (진입 또는 청산)
    pub fn handle_signal(&mut self, signal: &Signal, current_price: f64, current_date: &str) {
        if let PositionState::None = self.position {
            if let Signal::Buy { reason, initial_trailing_stop, take_profit, asset_pct } = signal {
                let trailing_stop_price = *initial_trailing_stop;

                self.position = PositionState::InPosition {
                    entry_price: current_price,
                    entry_asset: self.current_asset * asset_pct,
                    take_profit_price: *take_profit,
                    trailing_stop_price,
                };

                self.current_asset -= self.current_asset * asset_pct;

                println!("\x1b[34m[진입] 날짜: {}, 가격: {:.0}, 목표가: {:.0}, 트레일링스탑: {:.0}, 이유: {}\x1b[0m", 
                        current_date, current_price, take_profit, trailing_stop_price, reason);
            }
        }

        // 포지션이 있을 때, 매도 신호를 받으면 청산
        if let PositionState::InPosition { entry_price, entry_asset, .. } = self.position {
            if let Signal::Sell(reason) = signal {
                let pnl_pct = (current_price / entry_price - 1.0) - self.params.fees_pct * 2.0;
                println!("\x1b[35m[전략 매도] 날짜: {}, 진입가: {:.0}, 현재가: {:.0}, 실현 손익: {:.4}%, 이유: {}\x1b[0m", 
                        current_date,
                        entry_price, current_price, pnl_pct * 100.0, reason.reason);
                
                if pnl_pct > 0.0 {
                    self.win_count += 1;
                } else {
                    self.loss_count += 1;
                }

                self.current_asset += entry_asset * (1.0 + pnl_pct);
                self.total_pnl_pct = (self.current_asset / INITIAL_ASSET) - 1.0;
                self.position = PositionState::None; // 포지션 청산
                self.print_results(); // 중간 결과 출력
            }
            
            if let Signal::UpdateTrailingStop(new_trailing_stop) = signal {
                if let PositionState::InPosition { entry_price, entry_asset, take_profit_price, trailing_stop_price: _ } = self.position {
                    self.position = PositionState::InPosition {
                        entry_price,
                        entry_asset,
                        take_profit_price,
                        trailing_stop_price: *new_trailing_stop,
                    };
                }
            }
        }
    }

    /// 백테스팅 중간/최종 결과 출력
    pub fn print_results(&self) {
        let total_trades = self.win_count + self.loss_count;
        if total_trades == 0 { return; }

        let win_rate = (self.win_count as f64 / total_trades as f64) * 100.0;
        
        println!("--------------------------------------------------");
        println!(" [백테스팅 결과]");
        println!(" > 총 자산: {:.0}", self.current_asset);
        println!(" > 총 거래: {} 회 (승: {}, 패: {})", total_trades, self.win_count, self.loss_count);
        println!(" > 승률: {:.2}%", win_rate);
        println!(" > 총 누적 손익률: {:.4}%", self.total_pnl_pct * 100.0);
        println!("--------------------------------------------------");
    }
}

impl Add for BacktesterState {
    type Output = BacktesterState;

    fn add(self, rhs: BacktesterState) -> BacktesterState {
        BacktesterState {
            position: PositionState::None, // 합산 시 포지션은 의미 없음
            params: self.params,
            win_count: self.win_count + rhs.win_count,
            loss_count: self.loss_count + rhs.loss_count,
            total_pnl_pct: self.total_pnl_pct + rhs.total_pnl_pct,
            current_asset: self.current_asset + rhs.current_asset,
        }
    }
}

