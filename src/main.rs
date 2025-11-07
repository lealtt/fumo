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

use dotenv::dotenv;

mod commands;
mod config;
mod fumo;

pub use fumo::{Context, Data, Error};

#[tokio::main]
async fn main() -> Result<(), fumo::Error> {
    dotenv().ok();

    let token = config::discord_token()?;
    let intents = config::gateway_intents();
    let prefix_options = config::prefix_options();

    let framework = fumo::build_framework(prefix_options);
    fumo::run_client(token, intents, framework).await
}
