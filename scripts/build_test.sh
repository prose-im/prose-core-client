#!/bin/bash

# prose-core-client
#
# Copyright: 2022, Valerian Saliou <valerian@valeriansaliou.name>
# License: Mozilla Public License v2.0 (MPL v2.0)

export RUSTFLAGS="-L /path/to/libstrophe/lib/"
export TEST_JID="your-name@your-domain.com"
export TEST_PASSWORD="<your_secret_password>"

cargo run --example hello_bot
