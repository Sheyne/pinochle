FROM rust:latest AS builder
RUN apt-get update
RUN apt-get install musl-tools -y
RUN rustup target add x86_64-unknown-linux-musl
ADD lib/ lib/
ADD server/ server/
ADD server-logic/ server-logic/
RUN cd server && RUSTFLAGS=-Clinker=musl-gcc cargo build --release --bin pinochle-server --target=x86_64-unknown-linux-musl

FROM rust:latest AS client-builder
RUN apt-get update
RUN apt-get -y install npm
RUN npm install --global rollup
RUN rustup update
RUN rustup target add wasm32-unknown-unknown
RUN curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
ADD lib/ lib/
ADD client/ client/
RUN cd client && wasm-pack build --target web && rollup ./main.js --format iife --file ./pkg/bundle.js


FROM alpine:latest
COPY --from=builder /server/target/x86_64-unknown-linux-musl/release/pinochle-server /pinochle-server
COPY --from=client-builder /client/index.html /client/index.html
COPY --from=client-builder /client/style.css /client/style.css 
COPY --from=client-builder /client/pkg/bundle.js /client/pkg/bundle.js 
COPY --from=client-builder /client/pkg/pinochle_client_bg.wasm /client/pkg/pinochle_client_bg.wasm 
CMD ["/pinochle-server"]