# Spec Delta

## ADDED Requirements

### Requirement: Bounded Preview Retention
The GUI SHALL retain at most 100 completed wallpaper preview cache entries and at most 32 MiB of encoded preview bytes in its application-owned cache, using least-recently-used eviction. These budgets SHALL include completed asynchronous preview results and failure entries; they SHALL NOT require deleting the disk cache or evicting images currently used by visible widgets.

#### Scenario: Browse beyond the entry limit
- **WHEN** browsing additional wallpapers would retain more than 100 completed preview entries
- **THEN** the least-recently-used completed entries SHALL be evicted until the entry limit is satisfied
- **AND** recently accessed entries SHALL be retained ahead of older entries
- **AND** evicted previews SHALL remain reloadable using the existing disk preview cache

#### Scenario: Preview bytes exceed the budget
- **WHEN** a completed preview would cause retained encoded bytes to exceed 32 MiB
- **THEN** completed entries SHALL be evicted until the byte budget is satisfied
- **AND** a preview larger than the entire budget SHALL be delivered to its requester without being retained in the completed cache

#### Scenario: Concurrent requests for the same preview
- **WHEN** multiple callers request a preview while its load is already pending
- **THEN** they SHALL share the same pending load
- **AND** pending request tracking SHALL have an explicit finite bound
- **AND** reaching that bound SHALL NOT start an unbounded backlog of background preview work

#### Scenario: Library refresh with pending loads
- **WHEN** the library refresh clears its preview cache while preview loads are pending
- **THEN** late results from the previous cache generation SHALL NOT repopulate the cleared cache or replace a newer preview result
- **AND** a later request SHALL be able to load a fresh preview

#### Scenario: Failed preview loading
- **WHEN** preview loading fails
- **THEN** the GUI SHALL display its existing fallback
- **AND** repeated requests SHALL reuse a retained failure entry while it remains cached
- **AND** failure retention SHALL obey the entry budget

#### Scenario: Controller shutdown
- **WHEN** the controller shuts down
- **THEN** it SHALL release its cached preview references
- **AND** late completions SHALL NOT restore those references
