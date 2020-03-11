import init, { run_app } from './pkg/pinochle_client.js';
async function main() {
   await init('pkg/pinochle_client_bg.wasm');
   run_app();
}
main()