# Atlas Simulator

The **Atlas Simulator** is a modular, system-level simulator for the Atlas CPU family (Atlas8 / Atlas16 / Atlas32 / Atlas64).
It models a complete computer system — CPU, memory, buses, peripherals, and optional features like pipelines, caches, MMUs, and multi-core configurations — using a **dynamic, composable backend** designed to scale from simple microcontroller-style systems to complex SoCs.

## Goals

* Simulate full Atlas-based systems, not just individual CPU cores
* Support all Atlas architectures with a shared system model
* Enable **runtime system construction from configuration files**
* Modularly add or remove CPU/system features (caches, pipelines, MMU, multi-core)
* Cleanly separate CPU, buses, memory, and peripherals

## Architecture Overview

The simulator is built around **composable components and capabilities**:

* **BusDevice**
  Memory-mapped components such as RAM, ROM, UARTs, timers, and bus bridges

* **Bus**
  Responsible only for address decoding and request routing

* **BusMaster**
  Interface used by CPUs, DMA engines, and bus adapters

* **CPU Core and Capabilities**
  Each CPU is a stack of optional features:

  * Pipeline (optional)
  * Instruction / Data caches (optional)
  * MMU (optional)
  * Multi-core container (optional)

* **Bus Adapters**
  MMUs, caches, and bridges implemented as composable layers between CPU and bus

* **Interrupt Network**
  Modeled separately from memory buses for accurate hardware representation

This design allows each CPU/system to be **customized at runtime** without monolithic structs.

## Dynamic System Configuration

Systems are assembled from configuration files (TOML/YAML/JSON). Each CPU can specify:

* Type (Atlas8 / 16 / 32 / 64)
* Number of cores
* Pipeline usage
* Instruction/Data caches
* MMU

Example TOML:

```toml
[[cpu]]
type = "atlas8"
core_count = 1
pipeline = false
icache = false
dcache = false
mmu = false

[[cpu]]
type = "atlas32"
core_count = 2
pipeline = true
icache = true
dcache = true
mmu = true
```

The simulator dynamically wraps CPU cores with the appropriate adapters based on configuration, forming a fully connected system graph.

## Design Principles

1. CPUs are unaware of concrete devices
2. Buses only route addresses, they do not implement device logic
3. MMUs and caches are implemented as bus adapters
4. Pipelines are optional execution layers
5. Multi-core CPUs are containers of cores
6. Interrupts are modeled separately from the memory bus
7. Systems are fully composable and configurable at runtime

## Status

The simulator is under active development and evolves alongside the Atlas ISA, toolchain, and RTL implementations.
