FROM rust:1.78-alpine as builder
WORKDIR /usr/src/ucsc_menu
COPY . .
RUN apk add --no-cache musl-dev
# RUN apt update && apt install -y git pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
# RUN RUSTFLAGS="-Ctarget-feature=-crt-static" cargo install --path . 
RUN RUSTFLAGS="-Ctarget-feature=-crt-static" cargo install --path .

FROM alpine:latest
RUN apk add --no-cache musl libgcc
ENV PORT 8080
ENV HOST 0.0.0.0
COPY --from=builder /usr/local/cargo/bin/ucsc_menu /usr/local/bin/ucsc_menu
EXPOSE 8080
CMD ["ucsc_menu"]