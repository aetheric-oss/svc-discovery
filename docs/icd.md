# Interface Control Document (ICD) - `svc-discovery`

<center>

<img src="https://github.com/aetheric-oss/.github/blob/main/assets/doc-banner.png" style="height:250px" />

</center>

## Overview

This document defines the gRPC and REST interfaces unique to the `svc-discovery` microservice.

Attribute | Description
--- | ---
Status | Draft

## Related Documents

Document | Description
--- | ---
[High-Level Concept of Operations (CONOPS)](https://github.com/aetheric-oss/se-services/blob/develop/docs/conops.md) | Overview of Aetheric microservices.
[High-Level Interface Control Document (ICD)](https://github.com/aetheric-oss/se-services/blob/develop/docs/icd.md)  | Interfaces and frameworks common to all Aetheric microservices.
[Requirements - `svc-discovery`](https://nocodb.arrowair.com/dashboard/#/nc/view/ce00646b-1776-4a72-b01a-50dcd220de2a) | Requirements and user stories for this microservice.
[Concept of Operations - `svc-discovery`](./conops.md) | Defines the motivation and duties of this microservice.
[Software Design Document (SDD) - `svc-discovery`](./sdd.md) | Specifies the internal activity of this microservice.

## Frameworks

See the High-Level ICD.

## REST

See the High-Level ICD for common interfaces.


### Files

| File Location | Description |
--- | ---
`server/src/api_rest.rs` | Implements the REST endpoints.

### Authentication

See the High-Level ICD.

### Endpoints

| Endpoint | Type | Arguments | Description |
| ---- | --- | ---- | ---- |
| `/example` | GET | port_depart<br>port_arrive<br>time_range_start<br>time_range_end<br>cargo_weight_kg | This is an example REST endpoint.

## gRPC

### Files

These interfaces are defined in a protocol buffer file, `proto/grpc.proto`.

### Integrated Authentication & Encryption

See the High-Level ICD.

### gRPC Server Methods ("Services")

| Service | Description |
| ---- | ---- |
| `GetExample` | This is an example Service.<br>Replace

### gRPC Client Messages ("Requests")

| Request | Description |
| ------    | ------- |
| `ExampleQuery` | A message to illustrate an example
