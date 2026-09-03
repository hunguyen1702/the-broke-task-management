# Planning session handoff

## Mục tiêu session

Đi lần lượt qua từng product story trong [PRD](../PRD.md), phỏng vấn người dùng để chốt hướng implementation, rồi tạo hai đầu ra:

1. Mô tả story/epic trong `docs/epics/`.
2. Mô tả technical implementation task trong `docs/tasks/`.

Không implement source code trong planning session này.

## Trạng thái hiện tại

Story đã hoàn tất planning:

- **E1-S1: Initialize a repository**
  - [Epic plan](../epics/E1-S1-initialize-repository.md)
  - [Implementation task](../tasks/E1-S1-T1-implement-repository-initialization.md)
- **E1-S2: Resolve repository configuration**
  - [Epic plan](../epics/E1-S2-resolve-repository-configuration.md)
  - [Implementation task](../tasks/E1-S2-T1-implement-repository-configuration-resolution.md)
- **E1-S3: Register an agent**
  - [Epic plan](../epics/E1-S3-register-an-agent.md)
  - [Implementation task](../tasks/E1-S3-T1-implement-agent-registration.md)
- **E1-S4: List agents and claims**
  - [Epic plan](../epics/E1-S4-list-agents-and-claims.md)
  - [Implementation task](../tasks/E1-S4-T1-implement-agent-and-claim-listing.md)
- **E1-S5: Share repository state across Git worktrees**
  - [Epic plan](../epics/E1-S5-share-repository-state-across-git-worktrees.md)
  - [Implementation task](../tasks/E1-S5-T1-implement-shared-git-worktree-repository-resolution.md)
- **E2-S1: Create a task**
  - [Epic plan](../epics/E2-S1-create-a-task.md)
  - [Implementation task](../tasks/E2-S1-T1-implement-task-creation.md)
- **E2-S2: View and list tasks**
  - [Epic plan](../epics/E2-S2-view-and-list-tasks.md)
  - [Implementation task](../tasks/E2-S2-T1-implement-task-detail-and-listing.md)
- **E2-S3: Update task content**
  - [Epic plan](../epics/E2-S3-update-task-content.md)
  - [Implementation task](../tasks/E2-S3-T1-implement-task-content-updates.md)
- **E2-S4: Manage tags, URLs, estimates, and code references**
  - [Epic plan](../epics/E2-S4-manage-structured-task-context.md)
  - [Implementation task](../tasks/E2-S4-T1-implement-structured-task-context-updates.md)
- **E2-S5: Archive a task**
  - [Epic plan](../epics/E2-S5-archive-a-task.md)
  - [Implementation task](../tasks/E2-S5-T1-implement-safe-task-archival.md)
- **E2-S6: Unarchive a task safely**
  - [Epic plan](../epics/E2-S6-unarchive-a-task-safely.md)
  - [Implementation task](../tasks/E2-S6-T1-implement-safe-task-unarchive.md)
- **E3-S1: Use default statuses**
  - [Epic plan](../epics/E3-S1-use-default-statuses.md)
  - [Implementation task](../tasks/E3-S1-T1-implement-default-status-codes.md)
- **E4-S1: Manage task hierarchy**
  - [Epic plan](../epics/E4-S1-manage-task-hierarchy.md)
  - [Implementation task](../tasks/E4-S1-T1-implement-task-hierarchy-management.md)
- **E4-S2: Manage dependencies**
  - [Epic plan](../epics/E4-S2-manage-dependencies.md)
  - [Implementation task](../tasks/E4-S2-T1-implement-dependency-management.md)
- **E4-S3: Query available tasks**
  - [Epic plan](../epics/E4-S3-query-available-tasks.md)
  - [Implementation task](../tasks/E4-S3-T1-implement-available-task-querying.md)
- **E4-S4: Explain blocking**
  - [Epic plan](../epics/E4-S4-explain-blocking.md)
  - [Implementation task](../tasks/E4-S4-T1-implement-blocking-explanations.md)
- **E4-S5: View relationship maps**
  - [Epic plan](../epics/E4-S5-view-relationship-maps.md)
  - [Implementation task](../tasks/E4-S5-T1-implement-relationship-maps.md)
- **E5-S1: Claim a specified task atomically**
  - [Epic plan](../epics/E5-S1-claim-a-specified-task-atomically.md)
  - [Implementation task](../tasks/E5-S1-T1-implement-atomic-specified-task-claiming.md)
- **E5-S3: Unclaim owned work**
  - [Epic plan](../epics/E5-S3-unclaim-owned-work.md)
  - [Implementation task](../tasks/E5-S3-T1-implement-owner-controlled-task-unclaim.md)
- **E5-S2: Claim the next available task atomically**
  - [Epic plan](../epics/E5-S2-claim-the-next-available-task-atomically.md)
  - [Implementation task](../tasks/E5-S2-T1-implement-atomic-next-available-task-claiming.md)
- **E5-S4: Force-unclaim stale work**
  - [Epic plan](../epics/E5-S4-force-unclaim-stale-work.md)
  - [Implementation task](../tasks/E5-S4-T1-implement-user-force-unclaim.md)
- **E5-S5: Preserve claim/status independence**
  - [Epic plan](../epics/E5-S5-preserve-claim-status-independence.md)
  - [Implementation task](../tasks/E5-S5-T1-harden-claim-status-independence.md)
- **E6-S1: Add and view comments**
  - [Epic plan](../epics/E6-S1-add-and-view-comments.md)
  - [Implementation task](../tasks/E6-S1-T1-implement-task-comments.md)

Story được chủ đích defer:

- **E3-S2: Create and organize custom statuses** — default statuses đã đủ cho workflow MVP cơ bản; custom statuses không nằm trên critical path hiện tại.

Story tiếp theo nếu tiếp tục planning:

- **E6-S2: Delete a comment under ownership rules** — phụ thuộc E6-S1 implementation và sẽ chốt ownership failure/output behavior sau khi comment model hoàn tất.

### Implementation readiness tại thời điểm handoff

- **E1-S1-T1**, **E1-S2-T1**, **E1-S5-T1**, **E1-S3-T1**, **E3-S1-T1** và **E2-S1-T1** đã implementation xong; xem commit và dependency hiện hành trong [status dashboard](../STATUS.md).
- **E1-S4-T1** đã implementation xong với deterministic read-only agent và active-claim listing.
- **E3-S1-T1** đã hoàn tất tại commit `4d8ebb2`, cung cấp stable status codes và mở khóa task creation.
- **E2-S1-T1** đã hoàn tất tại commit `ca25528`, cung cấp task creation với explicit actor, atomic aggregate insert, stable task ID, status machine code và structured coding context.
- **E4-S2-T1** đã được implement và cung cấp authoritative dependency graph cho E5-S1 và E4-S3.
- **E4-S3-T1** đã implementation xong với authoritative available-task query.
- **E5-S1-T1** đã implementation xong, cung cấp authoritative active-claim model, atomic specified-task claim và claim hydration cho các story phụ thuộc.
- **E5-S3-T1** đã implementation xong với transactional owner-controlled unclaim behavior.
- **E5-S2-T1** đã implementation xong với atomic next-available-task selection và claiming.
- **E5-S4-T1** đã implementation xong với observed-claim confirmation binding, exact release và authoritative post-release availability.
- **E5-S5-T1** đã implementation xong với cross-command claim/status independence regression coverage.

E2-S5-T1, E2-S6-T1, E4-S1-T1, E4-S4-T1, E4-S5-T1, E5-S2-T1, E5-S3-T1, E5-S4-T1, E5-S5-T1, E4-S3-T1 và E1-S4-T1 đã implementation xong. E3-S2 tiếp tục được defer.

**E6-S1-T1** đã planning xong và `ready` trên E1-S3-T1 cùng E2-S1-T1; task này sẽ thêm comment creation/listing mà không thay đổi task mutation metadata.

## Việc đã làm

- Đọc toàn bộ PRD.
- Chốt E1-S1 qua nhiều vòng phỏng vấn và agent review.
- Chọn Rust làm ngôn ngữ CLI, quản lý toolchain bằng `mise`, binary tên `tbtm`, bundled SQLite.
- Chốt workspace tại `.tbtm/config.json` và `.tbtm/tbtm.db`.
- Chốt repository discovery, prefix normalization, default statuses, force-init backup, `--stealth`, JSON/human output và exit-code categories ở mức MVP.
- Thêm `tbtm uninstall` và `--dry-run` để người dùng dọn filesystem artifacts.
- Chủ đích giữ filesystem lifecycle đơn giản, best-effort: bước nào lỗi thì báo bước đó và artifacts đã tạo; người dùng tự quyết định uninstall rồi chạy lại. SQLite mutations vẫn transactional.
- Cập nhật PRD để phản ánh filesystem best-effort và uninstall.
- Viết và review epic/task E1-S1; review cuối kết luận READY cho implementation.
- Chốt E1-S2 với shared repository resolver, kiểm tra tối thiểu cho command thường và full read-only health check qua `tbtm repo status`.
- Chốt phân loại lỗi repository/config/database, schema-v1 database path cố định, JSON/human output và exit codes.
- Viết epic/task E1-S2; review planning kết luận READY trước khi tạo tài liệu.
- Chốt E1-S3 với đăng ký luôn tạo UUID mới, base-name dùng normalization của E1-S1, display name dùng suffix UUID tám ký tự và identity được truyền tường minh bằng `--agent <uuid>` cho các command sau.
- Chủ đích không lưu current-agent toàn cục để nhiều agent có thể dùng cùng repository đồng thời.
- Chốt `schemaVersion: 1` là compatibility version; internal migrations được theo dõi riêng, mutation command áp dụng pending compatible migrations còn `repo status` chỉ kiểm tra read-only.
- Viết epic/task E1-S3; review planning kết luận READY trước khi tạo tài liệu.
- Chốt E1-S4 với `tbtm agent list`, trả mọi agent kể cả không có claim, claim summary gồm task ID/title/status/claimed time, thứ tự ổn định và một read-only SQLite snapshot.
- E1-S4 được planned trước nhưng implementation chờ E5-S1 cùng task/status prerequisites; story không tự định nghĩa claim schema cạnh tranh.
- Viết epic/task E1-S4; review planning kết luận READY trước khi tạo tài liệu.
- Chốt E1-S5 với một canonical store tại `.tbtm` của main worktree. Mọi linked worktree resolve store đó qua trusted Git common metadata, không dùng symlink, redirect trong config, per-worktree store, hay legacy migration.
- Chốt output phân biệt `repositoryRoot` canonical và `worktreeRoot` hiện tại; init, force, stealth, uninstall, status và mọi resolver consumer phải được retrofit trong task E1-S5.
- Viết epic/task E1-S5; implementation task chủ động bao phủ code E1-S1/E1-S2 đã hoàn tất thay vì giả định các task cũ sẽ được chạy lại.
- Chốt E2-S1 với create nhận toàn bộ scalar và structured coding context nhưng chưa nhận parent/dependency/claim.
- Chốt actor mặc định `user`, explicit `--agent <uuid>`, immutable `createdBy`, và claim là quan hệ assignment duy nhất về sau.
- Chốt task ID `<prefix>-<type>-<8 hex UUID>`, priority mặc định 50, estimate theo giờ, atomic task aggregate, stable JSON và exit codes.
- Chốt stable status mapping `to_do` → `Todo`, `in_progress` → `In progress`, `done` → `Done`.
- Viết epic/task E2-S1; sau đó hoàn tất implementation tại commit `ca25528` khi E3-S1-T1 đã cung cấp status codes.
- Chốt E3-S1 là fresh-schema-only vì chưa có released repository format: sửa baseline migration `0001`, giữ latest migration `2`, không tạo migration/backfill cho development-only databases.
- Chốt code constraint, UUID identity, canonical seed values, transaction-friendly core lookup và không thêm CLI command/output trong E3-S1.
- Viết epic/task E3-S1; sau đó hoàn tất implementation tại commit `4d8ebb2`, mở khóa E2-S1-T1.
- Chốt E2-S2 với `task view` và `task list`, active-only mặc định, archived/all scopes, status/type/tag filters và deterministic priority/creation/ID ordering.
- Chuẩn hóa full-task output thành hierarchy/dependency objects và claim summary ổn định; E4/E5 sẽ populate và regression-test các field này khi authoritative models xuất hiện.
- Chốt compact list projection, direct-only relationship summaries, empty success, stable query errors và read-only one-snapshot behavior.
- Viết epic/task E2-S2; review planning kết luận READY. E2-S2-T1 sẵn sàng implementation sau E2-S1-T1.
- Chốt E2-S3 là partial patch cho title, Markdown context, type, status và priority; E2-S4 sở hữu tags, URLs, estimate và code references.
- Chốt type change giữ stable ID và được validate atomically với current parent cùng direct children; parent mutation vẫn thuộc E4-S1.
- Chốt actor metadata, valid no-op không ghi dữ liệu, archived-task rejection, full-detail success output và stable exit behavior.
- Viết epic/task E2-S3; review planning kết luận READY. E2-S3-T1 bị block đến khi E4-S1-T1 cung cấp authoritative hierarchy model.
- Chốt E2-S4 mở rộng `task update` bằng tri-state set/clear/omit cho estimate và replacement/clear/omit cho tags, URLs, code references.
- Chốt collection replacement giữ input order, reuse E2-S1 validators và exact duplicate behavior; clear/value conflicts đi qua shared error envelope.
- Chốt structured patch atomic, actor-aware, active-only và valid no-op không ghi dữ liệu; E2-S4 có thể dựng shared update path trước E2-S3 mà không phụ thuộc E4-S1.
- Viết epic/task E2-S4; review planning và review tài liệu đều kết luận READY. E2-S4-T1 sẵn sàng implementation trên E2-S1/E2-S2.
- Chốt E2-S5 với archive reason Markdown bắt buộc, active-claim guard, owner archive, logical-user force confirmation và atomic claim release/archive mutation.
- Chốt force confirmation bind với claimant ID và claim time đã hiển thị; claim thay thế trong lúc xác nhận không thể bị release âm thầm.
- Chốt `archiveReason` thuộc shared full-task contract, active task dùng `null`, archived task dùng reason không rỗng, compact list không thêm field và E2-S6 sẽ clear reason.
- Chốt same-reason re-archive là no-op giữ metadata, different-reason re-archive bị từ chối, cùng human-only cancellation output riêng.
- Cập nhật PRD và viết epic/task E2-S5; review planning và review tài liệu đều kết luận READY. E2-S5-T1 bị block bởi E4-S3-T1 và E5-S3-T1.
- Chốt E2-S6 là user-only unarchive, clear archive reason, giữ target unclaimed và không thay đổi downstream claims.
- Chốt impact chỉ gồm active incomplete direct dependents, chia deterministic thành claimed, otherwise-available và already-blocked-elsewhere; recursive-only descendants không tự nhận blocker mới.
- Chốt human/JSON confirmation, cancellation, active-task no-op và transaction recheck để không commit latest impact chưa từng được xác nhận.
- Chủ đích giữ concurrency MVP đơn giản: sau khi user xác nhận non-empty impact, transaction chấp nhận chi tiết thay đổi và trả latest impact, không dùng snapshot token hay `IMPACT_CHANGED`.
- Cập nhật PRD và viết epic/task E2-S6; planning review kết luận READY. E2-S6-T1 bị block bởi E2-S5-T1 và E4-S4-T1.
- Chủ đích defer E3-S2 vì ba default statuses đã đủ cho use case MVP cơ bản; ưu tiên dependency/claiming/availability critical path.
- Chốt E4-S2 với directed many-to-many mandatory dependencies, command `task dependency add/remove`, explicit downstream qua `<task-id>` và upstream qua `--depends-on`.
- Chốt chỉ downstream không archived mới được mutation; completed active downstream vẫn được sửa, archived upstream vẫn là target hợp lệ và thỏa edge theo effective completion.
- Chốt remove cạnh không tồn tại là `DEPENDENCY_NOT_FOUND`; duplicate, self và cycle đều bị từ chối; không expose arbitrary `cyclePath` ở MVP.
- Chốt mutation output tối giản: human xác nhận cạnh, JSON trả `{taskId, dependsOn}`; `task view` populate direct upstream/downstream summaries và compact list không đổi.
- Chốt graph validation/mutation/metadata atomic trong immediate transaction, không cache blocking/availability, và cung cấp shared all-upstreams-effective-completed predicate cho E4-S3/E5-S1.
- Viết epic/task E4-S2; planning review và review tài liệu kết luận READY sau khi bổ sung pending-migration behavior cho read-only task view. E4-S2-T1 sẵn sàng implementation.
- Chốt E4-S3 với `task available`, exact derived availability predicate và deterministic priority/creation/ID ordering.
- Chốt repeatable status/type/tag filters theo OR trong nhóm, AND giữa nhóm; known completed status trả empty success, unknown status và invalid type giữ shared errors.
- Chốt tái sử dụng compact E2-S2 projection, exact empty human/JSON output, không thêm availability hay blocker fields.
- Chốt read-only no-migration one-snapshot query và caller-owned connection/transaction selector để E5-S2 tái sử dụng nguyên predicate, filters và ordering trong atomic claim-next flow.
- Viết epic/task E4-S3; decision review và document review đều kết luận READY. E4-S3-T1 sẵn sàng implementation trên E3-S1-T1, E4-S2-T1 và E5-S1-T1.
- Chốt E5-S1 là agent-only command `task claim <task-id> --agent <uuid>`, không có implicit actor hoặc logical-user claim.
- Chốt exact availability recheck trong immediate transaction, một authoritative claim row cho mỗi task, same-agent re-claim vẫn là conflict và không cập nhật task status/mutation metadata.
- Chốt validation precedence ưu tiên existing claim trước archive/completion/dependency blocking; `CLAIM_CONFLICT` trả owner/timestamp và `TASK_NOT_AVAILABLE` dùng ba reason ổn định với unresolved upstream IDs được sort.
- Chốt contender quan sát winner trả deterministic conflict, còn busy-timeout trước khi quan sát ownership giữ operational database error trung thực.
- Viết epic/task E5-S1; decision review và document review đều kết luận READY. E5-S1-T1 sẵn sàng implementation trên E1-S3/E1-S5/E2-S1/E2-S2/E3-S1/E4-S2.
- Chốt E5-S3 là agent-only `task unclaim <task-id> --agent <uuid>`, chỉ current owner được release và missing claim là lỗi thay vì no-op.
- Chốt validation precedence agent trước task/claim/ownership; foreign owner trả claimant shape ổn định, claim deletion không đổi status hoặc task mutation metadata.
- Chốt success tái sử dụng full-task aggregate với `claim: null`, không thêm availability projection; derived availability được kiểm chứng qua claim behavior hiện có và E4-S3 sẽ sở hữu direct query output.
- Viết epic/task E5-S3; decision review và document review đều kết luận READY. E5-S3-T1 sẵn sàng implementation trên E5-S1-T1.
- Chốt E4-S4 với read-only `task blockers <task-id>`, trả đồng thời mọi reason archive/completion/claim/dependency theo thứ tự ổn định và không coi unavailable là command failure.
- Chốt unresolved dependency explanation thành direct và recursive collections: chỉ đi qua unresolved edges, dừng tại effective completion, direct thắng khi một node reachable ở nhiều depth, deduplicate/sort theo task ID và không traverse hierarchy.
- Chốt shared nested claim summary, deterministic human/JSON output, one-snapshot no-migration reads và caller-owned core query để E2-S6 tái sử dụng trong transaction.
- Viết epic/task E4-S4; decision review và document review đều kết luận READY.
- Chốt E4-S1 với child-oriented `task parent set/remove`; không tạo command add/remove child đối xứng cho cùng một cạnh.
- Chốt single-parent type matrix, complete-graph cycle validation và immediate transaction; cả child và parent phải active khi thay đổi cạnh, còn archive giữ nguyên hierarchy để bảo toàn lịch sử.
- Chốt same-parent set là valid no-op, missing-parent remove là `PARENT_NOT_FOUND`, child-only mutation metadata và stable validation/error precedence.
- Chốt `task hierarchy <task-id> [--recursive]` cho observable parent/direct-child/recursive-descendant reads; descendants sort theo depth rồi task ID, còn E4-S5 sở hữu combined relationship maps.
- Viết epic/task E4-S1; decision review và direct document review đều kết luận READY. E4-S1-T1 sẵn sàng implementation trên E2-S1-T1 và E2-S2-T1, sau đó mở khóa E2-S3-T1.
- Chốt E4-S5 với `task map <task-id> --direction`, mặc định `all`, và năm recursive mode upstream/downstream/parent/child/all.
- Chốt JSON canonical nodes + explicit edges, per-direction shortest-depth `reachedBy`, authoritative edge orientation, deterministic ordering, và cycle-safe bounded-query traversal không truncate.
- Chốt human output theo sectioned deterministic tree, exact UTF-8 connectors, shared-node references và badge ngắn `archived`, `completed`, `claimed`, `blocked`, `ready` với semantics độc lập.
- Viết epic/task E4-S5; decision review và direct document review đều kết luận READY. E4-S5-T1 sẵn sàng implementation trên E4-S1-T1, E4-S2-T1, E4-S3-T1 và E5-S1-T1.
- Chốt E5-S2 với agent-only `task claim-next`, filters và ordering tái sử dụng nguyên E4-S3, selection và E5-S1 claim insertion trong cùng immediate transaction.
- Chốt empty result là success (`data: null` / `No available task to claim.`), non-empty output tái sử dụng full-task và exact human renderer của E5-S1, không trả `CLAIM_CONFLICT`.
- Chốt writer concurrency được serialize trên canonical database: nhiều candidate cho distinct ordered claims, một candidate cho one claim plus one empty success; busy timeout là operational error.
- Chốt giữ SQLite `journal_mode = DELETE` cho MVP, không prompt hoặc external work trong locked section, không thêm retry/fallback hay thay đổi task metadata.
- Viết epic/task E5-S2; decision review kết luận READY. E5-S2-T1 sẵn sàng implementation trên E4-S3-T1 và E5-S1-T1.
- Chốt E5-S4 bằng cách mở rộng `task unclaim`: owner path tiếp tục dùng `--agent`, logical-user force path dùng `--force [--yes]`, và hai path loại trừ nhau.
- Chốt interactive confirmation bind với `{agentId, claimedAt}`; claim biến mất trả `CLAIM_NOT_FOUND`, claim thay thế trả `CLAIM_CHANGED`, và không bao giờ xóa claim chưa được quan sát.
- Chốt force-unclaim chỉ xóa claim, giữ nguyên task metadata, rồi trả wrapper `{task, releasedClaim, availability}` với authoritative reason precedence và deterministic human/JSON output.
- Ghi nhận hậu MVP về agent-runtime approval cho user-only commands trong `docs/improvements/agent-approval-for-user-only-commands.md` mà không biến logical actor thành authentication boundary.
- Viết và review epic/task E5-S4; decision review và direct document review đều kết luận READY. E5-S4-T1 sẵn sàng implementation.
- Chốt E5-S5 không thêm command, schema, migration, output hoặc error; story hợp nhất và regression-test các lifecycle contract hiện có.
- Chốt user, claim owner và foreign registered agent đều có thể đổi status của active claimed task; claim là coordination signal, không phải authorization lock.
- Chốt status transition hai chiều và valid no-op giữ nguyên claimant UUID/`claimedAt`; owner-unclaim giữ status, archive là ngoại lệ tự động release, và unarchive không phục hồi claim cũ.
- Viết epic/task E5-S5; decision review kết luận READY. E5-S5-T1 sẵn sàng implementation trên E3-S1/E2-S3/E5-S1/E5-S3/E2-S5/E2-S6.
- Chốt E6-S1 với `task comment add/list`, Markdown content bắt buộc không rỗng, UUID v4 comment ID và deterministic `createdAt`/ID chronological ordering.
- Chốt actor JSON là `user` hoặc registered-agent UUID; human output hydrate display name, và archived task nhận comment như active task.
- Chốt comment là child record độc lập, không đổi `updatedAt`, `updatedBy` hoặc state khác của task; comment history không được thêm vào shared full-task aggregate.
- Chốt add là transactional mutation có compatible migration, list là read-only one-snapshot/no-migration query, cùng stable human/JSON and exit behavior.
- Viết epic/task E6-S1; decision review kết luận READY. E6-S1-T1 sẵn sàng implementation trên E1-S3-T1 và E2-S1-T1.

## Quy tắc planning cho mỗi story

### 1. Đọc source requirement

- Đọc trực tiếp section của story trong `docs/PRD.md`; không dựa chỉ vào handoff này.
- Đọc thêm các section PRD được story tham chiếu: domain rules, functional requirements, workflows, validation, NFR, MVP acceptance criteria và dependency stories.
- Đọc epic/task đã planned nếu story mới phụ thuộc vào quyết định trước đó.
- PRD là source of truth cho product behavior; epic/task ghi solution decisions.

### 2. Tóm tắt context trước khi planning

- Trước khi bắt đầu phỏng vấn hoặc đề xuất hướng implementation, tóm tắt lại context của story cho người dùng.
- Tóm tắt phải nêu ngắn gọn outcome, behavior bắt buộc từ PRD, dependencies, các quyết định đã kế thừa từ story trước và những điểm còn mở cần planning.
- Chỉ bắt đầu câu hỏi planning sau khi context này đã được trình bày, để người dùng có cùng baseline khi đưa ra quyết định.

### 3. Phỏng vấn người dùng

- Đi từng story, bắt đầu từ story tiếp theo trong danh sách trạng thái.
- Chỉ hỏi điểm còn mơ hồ hoặc có nhiều hướng implementation ảnh hưởng observable behavior.
- Đề xuất phương án mặc định rõ ràng để người dùng có thể trả lời ngắn.
- Giữ MVP đơn giản; không đào sâu adversarial filesystem behavior, recovery protocol hoặc edge case hiếm nếu PRD không yêu cầu.
- Khi người dùng chốt trade-off khác PRD, nêu conflict và xin phép cập nhật PRD trước khi ghi plan.
- Không bắt đầu story kế tiếp trước khi story hiện tại đạt READY và tài liệu đã được tạo.

### 4. Review sau mỗi vòng trả lời

Sau mỗi lần người dùng trả lời câu hỏi planning, spawn một agent độc lập để đánh giá nội dung theo 5 tiêu chí:

1. Hướng implementation rõ ràng.
2. Có cách test.
3. Có cách verify đạt yêu cầu.
4. Có đủ reference cần thiết.
5. Có output và acceptance criteria đầy đủ, gồm functional và non-functional.

Agent review phải trả `READY` hoặc `NOT READY` và blocker cụ thể. Nếu `NOT READY`, tiếp tục phỏng vấn những blocker quan trọng. Không mở rộng sang chi tiết ít giá trị chỉ để đạt độ bao phủ lý thuyết.

Sau khi viết file, yêu cầu agent review trực tiếp PRD/epic/task để tìm contradiction quan trọng. Sửa blocker, rồi chạy review lại.

### 5. Tạo output

Khi READY, tạo:

```text
docs/epics/<STORY-ID>-<slug>.md
docs/tasks/<STORY-ID>-T1-<slug>.md
```

Epic plan nên gồm:

- Outcome và user story.
- Scope/out of scope.
- Product/solution decisions.
- Functional acceptance criteria.
- Non-functional acceptance criteria.
- Test/verification approach.
- Link tới implementation task.

Technical task nên gồm:

- Objective và deliverables.
- Proposed structure và technical choices.
- Implementation flow đủ để coding agent bắt đầu.
- Output/error contract cần thiết cho story.
- Test plan và verification commands.
- Definition of done.
- References.

Giữ plan đủ cụ thể để implement nhưng không biến thành line-by-line implementation hoặc catalogue mọi edge case.

### 6. Kiểm tra trước handoff

- Chạy `git diff --check`.
- Kiểm tra link giữa PRD, epic và task.
- Xác nhận PRD/epic/task không mâu thuẫn.
- Không sửa hoặc xóa unrelated user files.
- Mọi shell command phải prefix bằng `rtk` theo repository instruction.

## Trạng thái toàn bộ backlog

Xem dashboard tập trung tại [Planning and implementation status](../STATUS.md). Frontmatter trong từng epic/task document là nguồn trạng thái của document đó; dashboard bao gồm thêm các story vẫn cần planning.

## Cách tiếp tục

Nếu tiếp tục implementation:

1. Implement [E6-S1-T1: Implement task comments](../tasks/E6-S1-T1-implement-task-comments.md), sau khi claim trạng thái `ready` theo workflow repository.
2. Xem [status dashboard](../STATUS.md) trước khi claim task khác vì nhiều implementation session có thể đang dùng chung worktree.

Nếu tiếp tục planning:

1. Mở `docs/PRD.md` và tìm `Story E6-S2: Delete a comment under ownership rules`.
2. Đọc comment/actor/task rules, E6-S1 và các implementation contracts liên quan.
3. Chỉ bắt đầu planning E6-S2 sau khi E6-S1-T1 cung cấp authoritative comment schema và add/list behavior.
4. Sau mỗi câu trả lời, spawn review agent theo 5 tiêu chí.
5. Khi READY, tạo epic và technical task E6-S2, review file, cập nhật danh sách trong handoff này.
