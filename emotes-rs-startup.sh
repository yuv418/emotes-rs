#!/bin/sh

echo STARTING SERVER
/sqlx migrate run
/emotes-rs
