#!/bin/bash

mkdir -p certs
openssl genrsa -out certs/rsa-key.pem 2048
openssl req -new -x509 -key certs/rsa-key.pem -out certs/rsa-cert.pem -days 365 -subj "/CN=rsa-cert"
openssl ecparam -name prime256v1 -genkey -out certs/ecdsa-key.pem
openssl req -new -x509 -key certs/ecdsa-key.pem -out certs/ecdsa-cert.pem -days 365 -subj "/CN=ecdsa-cert"
