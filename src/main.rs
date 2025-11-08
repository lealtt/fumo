/*

Copyright 2025 Lealt

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.

*/

use dotenvy::dotenv;

mod commands;
mod constants;
mod database;
mod env;
mod fumo;
mod functions;

pub use fumo::{Context, Data, Error};

#[tokio::main]
async fn main() -> Result<(), fumo::Error> {
    dotenv().ok();

    let token = env::discord_token()?;
    let intents = fumo::gateway_intents();
    let prefix_options = fumo::prefix_options();
    let database = database::connect()
        .await
        .map_err(|err| -> fumo::Error { Box::new(err) })?;

    let framework = fumo::build_framework(prefix_options, database);
    fumo::run_client(token, intents, framework).await
}
