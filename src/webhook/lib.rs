use webhook::client::WebhookClient;
use chrono::Utc;

use crate::env_var;

const WEBHOOK_LOG_PREFIX: &str = "[Webhook]";
const WEBHOOK_USERNAME: &str = "ctb";

pub async fn send_webhook(title: &str, content: &str) {
    let url_env = env_var("WEBHOOK_URL");

    let url: &str = &url_env;
    let client: WebhookClient = WebhookClient::new(url);
    
    let result = client.send(|message| message
        .username(WEBHOOK_USERNAME)
        .embed(|embed| embed
            .title(title)
            .description(content)
        )
    ).await;

    match result {
        Ok(result) => {
            println!("{} Webhook sent successfully", WEBHOOK_LOG_PREFIX);
        }
        Err(e) => {
            println!("{} Failed to send webhook: {}", WEBHOOK_LOG_PREFIX, e);
        }
    }   
}

pub async fn send_buy_signal(
    symbol: &str,
    price: f64,
    volume: f64,
    stop_loss: f64,
    take_profit: f64,
    rr_ratio: f64,
    strategy: &str,
) {
    let url_env = env_var("WEBHOOK_URL");
    let url: &str = &url_env;
    let client: WebhookClient = WebhookClient::new(url);
    
    let current_time = Utc::now();
    let timestamp = current_time.format("%Y-%m-%d %H:%M:%S UTC").to_string();
    
    let profit_potential = ((take_profit - price) / price * 100.0);
    let loss_potential = ((price - stop_loss) / price * 100.0);
    
    let content = format!(
        "🟢 **{}** 매수 신호 감지!\n\n\
        💰 **매수가**: {:.4}\n\
        📊 **거래량**: {:.6}\n\
        🎯 **전략**: {}\n\
        🛑 **손절가**: {:.4} ({:.2}%)\n\
        🎯 **목표가**: {:.4} (+{:.2}%)\n\
        ⚖️ **R/R 비율**: {:.2}\n\
        📈 **예상 수익률**: {:.2}%\n\
        📉 **최대 손실률**: {:.2}%\n\n\
        ⏰ **시간**: {}\n\
        ",
        symbol, price, volume, strategy, stop_loss, loss_potential, 
        take_profit, profit_potential, rr_ratio, profit_potential, 
        loss_potential, timestamp
    );
    
    let result = client.send(|message| message
        .username(WEBHOOK_USERNAME)
        .embed(|embed| embed
            .title("매수 신호")
            .description(&content)
        )
    ).await;

    match result {
        Ok(_) => println!("{} 매수 신호 웹훅 전송 성공", WEBHOOK_LOG_PREFIX),
        Err(e) => println!("{} 매수 신호 웹훅 전송 실패: {}", WEBHOOK_LOG_PREFIX, e),
    }
}

pub async fn send_sell_signal(
    symbol: &str,
    price: f64,
    volume: f64,
    profit_loss: f64,
    profit_loss_percent: f64,
    strategy: &str,
    reason: &str,
) {
    let url_env = env_var("WEBHOOK_URL");
    let url: &str = &url_env;
    let client: WebhookClient = WebhookClient::new(url);
    
    let current_time = Utc::now();
    let timestamp = current_time.format("%Y-%m-%d %H:%M:%S UTC").to_string();
    
    // 수익/손실에 따른 이모지와 색상 결정
    let (emoji, color_emoji) = if profit_loss >= 0.0 {
        ("🟢", "🟢")
    } else {
        ("🔴", "🔴")
    };
    
    let content = format!(
        "{} **{}** 매도 신호 - {}\n\n\
        💰 **매도가**: {:.4}\n\
        📊 **거래량**: {:.6}\n\
        🎯 **전략**: {}\n\
        📈 **수익/손실**: {:.4} ({:+.2}%)\n\
        📝 **매도 사유**: {}\n\
        ⏰ **매도 시간**: {}\n\n\
        ",
        emoji, symbol, if profit_loss >= 0.0 { "수익 실현!" } else { "손절" },
        price, volume, strategy, profit_loss, profit_loss_percent, 
        reason, timestamp
    );
    
    let result = client.send(|message| message
        .username(WEBHOOK_USERNAME)
        .embed(|embed| embed
            .title("매도 신호")
            .description(&content)
        )
    ).await;

    match result {
        Ok(_) => println!("{} 매도 신호 웹훅 전송 성공", WEBHOOK_LOG_PREFIX),
        Err(e) => println!("{} 매도 신호 웹훅 전송 실패: {}", WEBHOOK_LOG_PREFIX, e),
    }
}

pub async fn send_trade_summary(
    symbol: &str,
    total_trades: i32,
    winning_trades: i32,
    total_profit: f64,
    win_rate: f64,
    avg_profit: f64,
    max_drawdown: f64,
) {
    let url_env = env_var("WEBHOOK_URL");
    let url: &str = &url_env;
    let client: WebhookClient = WebhookClient::new(url);
    
    let current_time = Utc::now();
    let timestamp = current_time.format("%Y-%m-%d %H:%M:%S UTC").to_string();
    
    let performance_emoji = if win_rate >= 60.0 { "🟢" } else if win_rate >= 40.0 { "🟡" } else { "🔴" };
    
    let content = format!(
        "📈 **{}** 거래 성과 요약 리포트\n\n\
        📊 **총 거래 수**: {}\n\
        ✅ **승리 거래**: {}\n\
        📈 **승률**: {:.1}%\n\
        💰 **총 수익**: {:.4}\n\
        📊 **평균 수익**: {:.4}\n\
        📉 **최대 손실**: {:.4}\n\n\
        {} **성과**: {}\n\
        ⏰ **생성 시간**: {}",
        symbol, total_trades, winning_trades, win_rate, 
        total_profit, avg_profit, max_drawdown,
        performance_emoji, if win_rate >= 60.0 { "우수" } else if win_rate >= 40.0 { "보통" } else { "개선 필요" },
        timestamp
    );
    
    let result = client.send(|message| message
        .username(WEBHOOK_USERNAME)
        .embed(|embed| embed
            .title("거래 성과 요약")
            .description(&content)
        )
    ).await;

    match result {
        Ok(_) => println!("{} 거래 요약 웹훅 전송 성공", WEBHOOK_LOG_PREFIX),
        Err(e) => println!("{} 거래 요약 웹훅 전송 실패: {}", WEBHOOK_LOG_PREFIX, e),
    }
}










