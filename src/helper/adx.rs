// 시가, 고가, 저가, 종가 데이터를 위한 구조체
#[derive(Debug, Clone, Copy)]
pub struct Ohlc {
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

// 최종 ADX 출력값을 위한 구조체
#[derive(Debug)]
pub struct AdxOutput {
    pub plus_di: f64,
    pub minus_di: f64,
    pub adx: f64,
}

/// ADX(Average Directional Index)
/// 추세의 강도를 측정하는 지표
/// 
/// 25 이상이면 강한 추세, 20 미만이면 약한 추세 또는 횡보장
pub fn calculate_adx(data: &[Ohlc], period: u32) -> Vec<AdxOutput> {
    let mut prev_ohlc: Option<Ohlc> = None;
    let mut smoothed_plus_dm = 0.0;
    let mut smoothed_minus_dm = 0.0;
    let mut smoothed_tr = 0.0;
    let mut dx_buffer: Vec<f64> = Vec::with_capacity(period as usize);
    let mut adx = 0.0;
    let mut is_warmed_up = false;
    let mut counter = 0u32;
    let mut results = Vec::with_capacity(data.len());

    for ohlc in data.iter() {
        counter += 1;
        let prev = match prev_ohlc {
            Some(p) => p,
            None => {
                prev_ohlc = Some(*ohlc);
                results.push(AdxOutput { plus_di: 0.0, minus_di: 0.0, adx: 0.0 });
                continue;
            }
        };
        let up_move = ohlc.high - prev.high;
        let down_move = prev.low - ohlc.low;
        let plus_dm = if up_move > down_move && up_move > 0.0 { up_move } else { 0.0 };
        let minus_dm = if down_move > up_move && down_move > 0.0 { down_move } else { 0.0 };
        let tr1 = ohlc.high - ohlc.low;
        let tr2 = (ohlc.high - prev.close).abs();
        let tr3 = (ohlc.low - prev.close).abs();
        let tr = tr1.max(tr2).max(tr3);
        if counter <= period {
            smoothed_plus_dm += plus_dm;
            smoothed_minus_dm += minus_dm;
            smoothed_tr += tr;
        } else {
            let period_f = period as f64;
            smoothed_plus_dm = smoothed_plus_dm - (smoothed_plus_dm / period_f) + plus_dm;
            smoothed_minus_dm = smoothed_minus_dm - (smoothed_minus_dm / period_f) + minus_dm;
            smoothed_tr = smoothed_tr - (smoothed_tr / period_f) + tr;
        }
        prev_ohlc = Some(*ohlc);
        if counter < period {
            results.push(AdxOutput { plus_di: 0.0, minus_di: 0.0, adx: 0.0 });
            continue;
        }
        if smoothed_tr == 0.0 {
            results.push(AdxOutput { plus_di: 0.0, minus_di: 0.0, adx });
            continue;
        }
        let plus_di = (smoothed_plus_dm / smoothed_tr) * 100.0;
        let minus_di = (smoothed_minus_dm / smoothed_tr) * 100.0;
        let di_sum = plus_di + minus_di;
        let dx = if di_sum == 0.0 { 0.0 } else { (plus_di - minus_di).abs() / di_sum * 100.0 };
        if !is_warmed_up {
            dx_buffer.push(dx);
            if dx_buffer.len() == period as usize {
                adx = dx_buffer.iter().sum::<f64>() / period as f64;
                is_warmed_up = true;
            }
        } else {
            let period_f = period as f64;
            adx = (adx * (period_f - 1.0) + dx) / period_f;
        }
        if is_warmed_up {
            results.push(AdxOutput { plus_di, minus_di, adx });
        } else {
            results.push(AdxOutput { plus_di: 0.0, minus_di: 0.0, adx: 0.0 });
        }
    }
    results
}