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
    update_interval_in_seconds: u64,
}

impl StateTrackerClient {
    pub fn new(
        id: String,
        state_sender: tokio::sync::mpsc::Sender<TrackedData>,
        update_interval_in_seconds: u64,
    ) -> StateTrackerClient {
        StateTrackerClient {
            id,
            state_sender,
            latest_update: Instant::now(),
            update_interval_in_seconds,
        }
    }

    pub async fn send_state(&self, state: State) -> Result<(), Error> {
        // Avoid spamming Idle & Valid states.
        if !state.is_error()
            && self.latest_update.elapsed().as_secs() < self.update_interval_in_seconds
        {
            return Ok(());
        }

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

#[cfg(test)]
#[tokio::test]
pub async fn avoids_spamming_idle_and_active_states() {
    const ID: &str = "ID";
    const UPDATE_INTERVAL_IN_SECONDS: u64 = 5;

    let (state_sender, mut state_receiver) = tokio::sync::mpsc::channel::<TrackedData>(5);

    let state_tracker_client =
        StateTrackerClient::new(ID.to_string(), state_sender, UPDATE_INTERVAL_IN_SECONDS);

    state_tracker_client.send_state(State::Valid).await.unwrap();

    match state_receiver.try_recv() {
        Ok(_) => panic!("should not have received a state"),
        Err(error) => assert_eq!(error, tokio::sync::mpsc::error::TryRecvError::Empty),
    }
}

#[tokio::test]
pub async fn error_state_is_instantly_set() {
    const ID: &str = "ID";
    const UPDATE_INTERVAL_IN_SECONDS: u64 = 5;
    const ERROR_MESSAGE: &str = "TEST_ERROR";

    let (state_sender, mut state_receiver) = tokio::sync::mpsc::channel::<TrackedData>(5);

    let state_tracker_client =
        StateTrackerClient::new(ID.to_string(), state_sender, UPDATE_INTERVAL_IN_SECONDS);

    state_tracker_client
        .send_state(State::Error(ERROR_MESSAGE.to_string()))
        .await
        .unwrap();

    match state_receiver.try_recv() {
        Ok(tracked_data) => match tracked_data.state {
            State::Error(_) => {
                assert_eq!(tracked_data.id, ID);
                assert_eq!(tracked_data.state, State::Error(ERROR_MESSAGE.to_string()));
            }
            _ => panic!("should have received an error state"),
        },
        Err(error) => panic!("should have received a state"),
    }
}
