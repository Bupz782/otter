use domain::models::condition::Metric;
use domain::models::intent::{Asset, ConditionalIntent};
use tokio::sync::mpsc::{self, error::TrySendError};

/// Events flowing through the system.
///
/// The event bus is intentionally small for the MVP. New variants will be added
/// as the orchestrator gains monitoring, proving, and submission stages.
#[derive(Debug, Clone, serde::Serialize)]
pub enum Event {
    /// A new market-data sample is available for an asset.
    PriceUpdated {
        asset: Asset,
        metric: Metric,
        value: u128,
    },
    /// A monitored intent's condition has been satisfied.
    ConditionMet { intent_id: String },
    /// A natural-language intent has been parsed successfully.
    IntentParsed {
        intent_id: String,
        conditional: ConditionalIntent,
    },
    /// A zero-knowledge proof has been generated for an intent.
    ProofGenerated {
        intent_id: String,
        proof_hash: String,
    },
    /// A transaction has been submitted on-chain.
    TransactionSubmitted { intent_id: String, tx_hash: String },
    /// A transaction has been confirmed on-chain.
    TransactionConfirmed {
        intent_id: String,
        receipt: String,
        gas_used: u64,
    },
    /// A non-recoverable error occurred in the pipeline.
    Error { source: String, message: String },
}

/// Simple single-producer / multi-consumer event bus.
///
/// Under the hood this uses a bounded `tokio::sync::mpsc` channel. The
/// orchestrator owns the receiver; publishers clone the sender.
#[derive(Debug, Clone)]
pub struct EventBus {
    sender: mpsc::Sender<Event>,
}

impl EventBus {
    /// Create a new bus with the given channel capacity.
    pub fn new(capacity: usize) -> (Self, mpsc::Receiver<Event>) {
        let (sender, receiver) = mpsc::channel(capacity);
        (Self { sender }, receiver)
    }

    /// Publish an event to all subscribers.
    #[allow(clippy::result_large_err)]
    pub fn publish(&self, event: Event) -> Result<(), TrySendError<Event>> {
        self.sender.try_send(event)
    }

    /// Obtain a sender handle that can be used to publish events.
    pub fn publisher(&self) -> mpsc::Sender<Event> {
        self.sender.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn event_bus_delivers_events() {
        let (bus, mut receiver) = EventBus::new(16);
        bus.publish(Event::PriceUpdated {
            asset: Asset::Eth,
            metric: Metric::Price,
            value: 2_000_000_000,
        })
        .unwrap();

        let event = receiver.recv().await.expect("event should be received");
        assert!(matches!(
            event,
            Event::PriceUpdated {
                asset: Asset::Eth,
                metric: Metric::Price,
                value: 2_000_000_000,
            }
        ));
    }
}
