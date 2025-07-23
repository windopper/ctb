pub struct ParabolicSarCandle {
    pub high: f64,
    pub low: f64,
}

/// 파라볼릭 sar
/// 추세의 방향을 알려주고 잠재적인 반전 지점을 식별하는 데 사용되는 기술적 분석 지표
///
pub fn calculate_parabolic_sar(
    candles: &Vec<ParabolicSarCandle>,
    initial_af: f64,
    max_af: f64,
    af_increment: f64,
) -> Vec<f64> {
    let mut sars = Vec::with_capacity(candles.len());
    if candles.is_empty() {
        return sars;
    }

    let mut sar = candles[0].low;
    let mut ep = candles[0].high;
    let mut af = initial_af;
    let mut is_rising = true;

    sars.push(sar);

    for i in 1..candles.len() {
        let high = candles[i].high;
        let low = candles[i].low;
        let mut next_sar = sar;

        if is_rising {
            next_sar = sar + af * (ep - sar);
            if high > ep {
                ep = high;
                af = (af + af_increment).min(max_af);
            }
            if next_sar > low {
                // 추세 반전
                is_rising = false;
                next_sar = ep;
                ep = low;
                af = initial_af;
            }
        } else {
            next_sar = sar - af * (sar - ep);
            if low < ep {
                ep = low;
                af = (af + af_increment).min(max_af);
            }
            if next_sar < high {
                // 추세 반전
                is_rising = true;
                next_sar = ep;
                ep = high;
                af = initial_af;
            }
        }
        sar = next_sar;
        sars.push(sar);
    }
    sars
}