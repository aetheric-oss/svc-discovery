# Software Design Document (SDD) - `svc-discovery` 

<center>

<img src="https://github.com/aetheric-oss/.github/blob/main/assets/doc-banner.png" style="height:250px" />

</center>

## Overview

This document details the software implementation of svc-discovery.

This microservice is the API for other service providers (U-space service providers (USSPs) as an example) to interact with the Aetheric network. Likewise, it also allows our network to make requests to external service providers to allow users to access vertiports outside of our network.

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
[Interface Control Document (ICD) - `svc-discovery`](./icd.md) | Defines the inputs and outputs of this microservice.

## Module Attributes

Attribute | Applies | Explanation
--- | --- | ---
Safety Critical | ? | 
Realtime | ? |

## Logic

### Initialization

FIXME Description of activities at init

### Loop

FIXME Description of activities during loop

### Cleanup

FIXME Description of activities at cleanup

## Interface Handlers

FIXME - What internal activities are triggered by messages at this module's interfaces?
