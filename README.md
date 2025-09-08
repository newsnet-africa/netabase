# Netabase.rs

This crate is intended to provide an *Object store abstraction* (object mapper?) for libp2p's implementation of the Kademlia DHT. By v0.0.1, this should enable the following functionality/abstractions:
- [ ] Serialisation/Deserialsation
- [ ] Basic CRUD abstractions
- [ ] Provide Kademlia defaults and configuration api
- [ ] Handle management of a running swarm thread/event loop through message passing

- Note that this crate is designed for `NewsNet`, so some features might be specific for that use case.
- Right now, I still need to complete the implementation/re-re-refactor of the main API, but eventually, this crate should be usable for general purpose decentralisation networks.
- Until the main features of the `TODO` list are implemented, testing is unimplemented as there are often big changes that require new tests all the time. I will make an effort to add examples for the incremental versions, but note that these are for sure going to be outdated by like, the next release.

## TODO:
### v0.0.1:
	- [ ] General code cleanup:
		- [ ] Optimisations:
			- [ ] Reduce cloning
			- [ ] Refactor structure for modularity:
				- [ ] Create optional features
	- [ ] Documentation:
		- [ ] Module level docs
		- [ ] Main api
	- [ ] Macro library :
		- [ ] Generate serialisation and deserialisation implementations for Schemas:
			- [x] Bincode
			- [ ] Serde
		- [ ] Generate conversion to/from kademlia record types:
			- [x] Infallable From/Into implementations
			- [ ] Fallable TryFrom/TryInto implementations:
				- [ ] Replace From/Into
				- [ ] Create conversion result type
		- [ ] General code cleanup:
			- [ ] Refactor visitors and generators to reduce coupling
				- [ ] Create `DeriveItem` lifetimes and manage parsing accordingly
				- [ ] AST lifetimes
			- [ ] Clean up imports/exports and general dependency management
			- [ ] Handle error cases and other `todo()!` and `"Fix later"` exceptions
			- [ ] Panic and compile error documentation for validation reasons

	- [ ] Core library:
		- [ ] Consolidate the `Netabase` object:
			- [ ] Create (default) config structs:
				- [ ] Kademlia
					- [ ] Server and client control abstractions
					- [ ] Network protection configuration
				- [ ] Identify
				- [ ] mDNS
				- [ ] Swarm
					- [ ] Listen Addresses
					- [ ] Protocol (?)
		- [ ] Swarm creation and management:
			- [ ] Message passing:
				- [ ] Swarm Commands
				- [ ] Network Events 

### v0.0.2:
	- [ ] Query functionality:
		- [ ] Implement `rustqlite` (or some other library) layer
		- [ ] Implement gossipsub query functionality:
			- [ ] Create/implement some sort of GraphQL for querying on gossipsub
			- [ ] Design message flow for query:
				- Do we return data, or batch pointers?
				- Load management
	- [ ] Persistence:
		- [ ] Design incentive model for data persistence
		- [ ] Implement optional dedicated provider nodes for larger network implementations
		- [ ] Implement timing/rule based toggle for client/server switching
	- [ ] Graph query design (Like MongoDB):
		- [ ] On demand child field queries

## Goals for v1.0.0:
	- [ ] Queriability over network
	- [ ] Custom functions/behaviours for network events
	- [ ] The usual OSA functionality (CRUD)
