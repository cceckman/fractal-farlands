/// User-interface rendering for Fractal Farlands.

use axum::extract::Query;
use crate::WindowParams;

/// Render the user interface.
pub async fn interface(query: Query<WindowParams>) -> String {
    format!("Provided parameters: {:?}", &query)
}

