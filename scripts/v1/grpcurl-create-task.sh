#!/bin/bash

grpcurl -plaintext \
  -import-path ./proto \
  -proto ./proto/todos/v1/todos.proto \
  -d '{"story_id": "5f99f1fb-8410-41bb-8e0b-56c05350d736", "name": "No Country for Old Men"}' \
  "[::]:9090" \
  todos.v1.TodosService/CreateTask
