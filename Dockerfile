FROM rust:1.78-alpine as builder
WORKDIR /usr/src/ucsc_menu
COPY . .
RUN cargo install --path .

FROM alpine:latest
# RUN apt-cache search libssl3-dev
# RUN apt update && apt install -y build-essential libssl-dev && rm -rf /var/lib/apt/lists/*
# RUN ldconfig /usr/local/lib64/
COPY --from=builder /usr/local/cargo/bin/ucsc_menu /usr/local/bin/ucsc_menu
CMD ["ucsc_menu"]