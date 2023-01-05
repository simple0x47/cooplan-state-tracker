use crate::error::{Error, ErrorKind};
use crate::state::State;
use crate::tracked_data;
use crate::tracked_data::TrackedData;
use tokio::time::Instant;

#[derive(Clone)]
pub struct StateTrackerClient {
    id: String,
    state_sender: tokio::sync::mpsc::Sender<TrackedData>,
    latest_update: Instant,
    update_interval_in_seconds: u32,
}

impl StateTrackerClient {
    pub fn new(
        id: String,
        state_sender: tokio::sync::mpsc::Sender<TrackedData>,
        update_interval_in_seconds: u32,
    ) -> StateTrackerClient {
        StateTrackerClient {
            id,
            state_sender,
            latest_update: Instant::now(),
            update_interval_in_seconds,
        }
    }

    pub async fn send_state(&self, state: State) -> Result<(), Error> {
        let tracked_data = tracked_data::generate_state_tracking_data(&self.id, state);

        match self.state_sender.send(tracked_data).await {
            Ok(_) => (),
            Err(error) => {
                return Err(Error::new(
                    ErrorKind::InternalFailure,
                    format!("failed to send state to state tracker: {}", error),
                ))
            }
        }

        Ok(())
    }
}
