use crate::core::signal::Signal;

#[derive(Clone, Copy)]
pub struct BacktestParams {
    pub take_profit_pct: f64, // 익절 비율 (예: 0.005는 0.5%)
    pub stop_loss_pct: f64,   // 손절 비율 (예: 0.003는 0.3%)
    pub trailing_stop_pct: f64, // 트레일링 스탑 비율 (예: 0.004는 0.4%)
    pub fees_pct: f64,        // 매수/매도 수수료
}

// 가상 포지션의 상태
#[derive(Debug, Clone, Copy)]
pub enum PositionState {
    None, // 포지션 없음
    InPosition {
        entry_price: f64,
        take_profit_price: f64,
        stop_loss_price: f64,
        trailing_stop_price: f64, // 트레일링 스탑 가격
    },
}

// 백테스터의 전체 상태와 결과를 관리
pub struct BacktesterState {
    position: PositionState,
    params: BacktestParams,
    // --- 성과 측정 지표 ---
    win_count: u32,
    loss_count: u32,
    total_pnl_pct: f64, // 총 누적 손익률
}

impl BacktesterState {
    pub fn new(params: BacktestParams) -> Self {
        BacktesterState {
            position: PositionState::None,
            params,
            win_count: 0,
            loss_count: 0,
            total_pnl_pct: 0.0,
        }
    }
    
    /// 매 프레임마다 현재 가격을 체크하여 포지션을 청산할 지 결정
    pub fn check_and_close_position(&mut self, current_price: f64) {
        if let PositionState::InPosition { entry_price, take_profit_price, stop_loss_price, mut trailing_stop_price } = self.position {
            let mut pnl_pct = 0.0;
            let mut closed = false;

            // 트레일링 스탑 가격 갱신 (현재가가 진입가보다 높을 때만)
            if current_price > entry_price {
                let new_trailing = current_price * (1.0 - self.params.trailing_stop_pct);
                if new_trailing > trailing_stop_price {
                    trailing_stop_price = new_trailing;
                }
            }

            // 익절 조건
            if current_price >= take_profit_price {
                self.win_count += 1;
                pnl_pct = (take_profit_price / entry_price - 1.0) - self.params.fees_pct * 2.0;
                println!("\x1b[32m[익절] 진입가: {:.0}, 목표가: {:.0}, 현재가: {:.0}, 수익률: {:.4}%\x1b[0m", 
                entry_price, take_profit_price, current_price, pnl_pct * 100.0);
                closed = true;
            }
            // 트레일링 스탑 or 손절 조건
            else if current_price <= trailing_stop_price || current_price <= stop_loss_price {
                self.loss_count += 1;
                let exit_price = if current_price <= trailing_stop_price { trailing_stop_price } else { stop_loss_price };
                pnl_pct = (exit_price / entry_price - 1.0) - self.params.fees_pct * 2.0;
                println!("\x1b[31m[손절] 진입가: {:.0}, 손절가: {:.0}, 현재가: {:.0}, 손실률: {:.4}% (트레일링: {:.0})\x1b[0m", 
                            entry_price, stop_loss_price, current_price, pnl_pct * 100.0, trailing_stop_price);
                closed = true;
            }

            if closed {
                self.total_pnl_pct += pnl_pct;
                self.position = PositionState::None; // 포지션 청산
                self.print_results(); // 중간 결과 출력
            } else {
                // 포지션 상태 갱신 (트레일링 스탑 가격 반영)
                self.position = PositionState::InPosition {
                    entry_price,
                    take_profit_price,
                    stop_loss_price,
                    trailing_stop_price,
                };
            }
        }
    }

    /// 전략 신호에 따라 포지션을 관리 (진입 또는 청산)
    pub fn handle_signal(&mut self, signal: Signal, current_price: f64) {
        if let PositionState::None = self.position {
            if signal == Signal::Buy {
                let take_profit_price = current_price * (1.0 + self.params.take_profit_pct);
                let stop_loss_price = current_price * (1.0 - self.params.stop_loss_pct);
                let trailing_stop_price = stop_loss_price; // 최초 진입 시 트레일링 스탑은 손절가와 동일

                self.position = PositionState::InPosition {
                    entry_price: current_price,
                    take_profit_price,
                    stop_loss_price,
                    trailing_stop_price,
                };
                println!("\x1b[34m[진입] 가격: {:.0}, 목표가: {:.0}, 손절가: {:.0}, 트레일링스탑: {:.0}\x1b[0m", 
                        current_price, take_profit_price, stop_loss_price, trailing_stop_price);
            }
        }

        // 포지션이 있을 때, 매도 신호를 받으면 청산
        if let PositionState::InPosition { entry_price, .. } = self.position {
            if signal == Signal::Sell {
                let pnl_pct = (current_price / entry_price - 1.0) - self.params.fees_pct * 2.0;
                println!("\x1b[35m[전략 매도] 진입가: {:.0}, 현재가: {:.0}, 실현 손익: {:.4}%\x1b[0m", 
                        entry_price, current_price, pnl_pct * 100.0);
                
                if pnl_pct > 0.0 {
                    self.win_count += 1;
                } else {
                    self.loss_count += 1;
                }

                self.total_pnl_pct += pnl_pct;
                self.position = PositionState::None; // 포지션 청산
                self.print_results(); // 중간 결과 출력
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
        println!(" > 총 거래: {} 회 (승: {}, 패: {})", total_trades, self.win_count, self.loss_count);
        println!(" > 승률: {:.2}%", win_rate);
        println!(" > 총 누적 손익률: {:.4}%", self.total_pnl_pct * 100.0);
        println!("--------------------------------------------------");
    }
}

