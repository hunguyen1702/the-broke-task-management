# Planning session handoff

## Mục tiêu session

Planning các product story đủ điều kiện trong [PRD](../PRD.md), phỏng vấn người dùng để chốt hướng implementation, rồi tạo hai đầu ra cho mỗi story:

1. Mô tả story/epic trong `docs/epics/`.
2. Mô tả technical implementation task trong `docs/tasks/`.

Không implement source code trong planning session này.

Nhiều story có thể được planning song song khi thỏa contract dependency và
quy tắc ownership bên dưới. Parallel planning không cho phép một story consumer
đi trước contract mà nó kế thừa.

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
- **E3-S2: Create and organize custom statuses**
  - [Epic plan](../epics/E3-S2-create-and-organize-custom-statuses.md)
  - [Implementation task](../tasks/E3-S2-T1-implement-custom-status-creation-and-ordering.md)
- **E3-S3: Change status completion semantics**
  - [Epic plan](../epics/E3-S3-change-status-completion-semantics.md)
  - [Implementation task](../tasks/E3-S3-T1-implement-repository-status-completion-changes.md)
- **E3-S4: Delete an unused custom status**
  - [Epic plan](../epics/E3-S4-delete-an-unused-custom-status.md)
  - [Implementation task](../tasks/E3-S4-T1-implement-safe-custom-status-deletion.md)
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
- **E6-S2: Delete comments under ownership rules**
  - [Epic plan](../epics/E6-S2-delete-comments-under-ownership-rules.md)
  - [Implementation task](../tasks/E6-S2-T1-implement-ownership-aware-comment-deletion.md)
- **E6-S3: Keep comments immutable**
  - [Epic plan](../epics/E6-S3-keep-comments-immutable.md)
  - [Implementation task](../tasks/E6-S3-T1-harden-comment-immutability.md)
- **E7-S1: Provide consistent command output**
  - [Epic plan](../epics/E7-S1-provide-consistent-command-output.md)
  - [Implementation task](../tasks/E7-S1-T1-standardize-cli-output-boundary.md)
- **E7-S2: Expose repository, agent, and task operations**
  - [Epic plan](../epics/E7-S2-expose-repository-agent-and-task-operations.md)
  - [Implementation task](../tasks/E7-S2-T1-harden-repository-agent-and-task-cli-surface.md)
- **E7-S3: Expose planning relationships and status operations**
  - [Epic plan](../epics/E7-S3-expose-planning-relationships-and-status-operations.md)
  - [Implementation task](../tasks/E7-S3-T1-harden-planning-relationship-and-status-cli-surface.md)
- **E7-S4: Expose availability and claim operations**
  - [Epic plan](../epics/E7-S4-expose-availability-and-claim-operations.md)
  - [Implementation task](../tasks/E7-S4-T1-harden-availability-and-claim-cli-surface.md)
- **E7-S5: Expose comment operations**
  - [Epic plan](../epics/E7-S5-expose-comment-operations.md)
  - [Implementation task](../tasks/E7-S5-T1-harden-comment-cli-surface.md)
- **E9-S1: Verify concurrent claim safety**
  - [Epic plan](../epics/E9-S1-verify-concurrent-claim-safety.md)
  - [Implementation task](../tasks/E9-S1-T1-harden-concurrent-claim-verification.md)

Planning tiếp theo:

- **H1-T0** đã xác nhận flow node-first và hoàn tất planning.
- **H1-T1 Constitution** đã có [implementation plan](../tasks/H1-T1-implement-constitution.md)
  và đã implementation xong. Năm điểm cuối (schema, exact revision references,
  approval của framework ruleset, scope, validator CLI) được chốt trong task
  contract; không áp schema đó cho các node khác.
- **H1-T11 Current Truth** đã có [implementation plan](../tasks/H1-T11-current-truth.md)
  và đã implementation xong. Node nhận information need đủ cụ thể từ Intent,
  tạo view ngắn có dẫn nguồn hoặc kết quả unresolved; không chọn route, không
  tạo ADR store thứ hai. Review cuối `READY`; cross-check H1-T0/H1-T1:
  `NO CONFLICT`.
- **H1-T12 Intent** đã hoàn tất planning trong
  [implementation plan](../tasks/H1-T12-intent.md) và đã implementation xong.
  `AGENTS.md` chỉ là entry tra cứu; Constitution rule kích hoạt Intent khi bắt
  đầu công việc mới. Intent trả `needs_clarification` hoặc `ready`, hiển thị
  diễn giải Context–Task–Format và chờ xác nhận rõ ràng trước khi chuyển sang
  Current Truth. H1 và H1-T1 ghi ngoại lệ xác nhận intent bắt buộc cùng phạm
  vi cho phép thêm framework workflow rule trong giai đoạn xây dựng; manifest,
  digest, index và validator vẫn phải nhất quán. Review cuối `READY`;
  cross-check H1-T0/H1-T1/H1-T11: `NO CONFLICT`. Bổ sung yêu cầu flowchart,
  diễn giải ngắn gọn và ví dụ bad/good cho logic đánh giá cùng output. Hai
  workflow rule đã active trong Constitution 1.2.0; acceptance impact `none`.
- **H1-T13 Risk Router** đã hoàn tất planning trong
  [implementation plan](../tasks/H1-T13-risk-router.md), sẵn sàng
  implementation. Sáu câu hỏi là tín hiệu định tính cho agent suy luận,
  không phải tổng điểm tự động. Output mỗi lần route chỉ là path, lý do ngắn,
  và bước tiếp theo, hiển thị cho user và chuyển cho workflow kế tiếp trong
  hội thoại; không tạo document bắt buộc. Các route giữ flow H1-T0; chỉ goal
  hoặc scope thay đổi đáng kể mới quay lại Intent. Decision review và direct
  document review: READY; đọc lại H1-T0/H1-T1/H1-T11/H1-T12: NO CONFLICT.
- **H1-T14 Direct path** đã hoàn tất planning trong
  [implementation plan](../tasks/H1-T14-direct-path.md) và sẵn sàng
  implementation. Node recheck các giả định của Router, frame action
  bounded dạng ephemeral với constraint, provisional proof và stop condition,
  rồi kết thúc tại handoff sang Execute; không thêm approval gate thông
  thường, mini-plan bền vững hay định nghĩa trước Execute/Proof of Work.
  Independent review: READY; đọc lại H1-T0/H1-T1/H1-T11/H1-T12/H1-T13:
  NO CONFLICT.
- **H1-T15 Research** đã hoàn tất planning trong
  [implementation plan](../tasks/H1-T15-research.md), implementation `ready`.
  User đã xác nhận lookup read-only có giới hạn; Context Capsule mặc định
  trong hội thoại, chỉ lưu khi có nhu cầu tái sử dụng rõ ràng hoặc user yêu cầu.
  Kết quả có nguồn hoặc gap chưa giải quyết quay qua Current Truth về Router;
  không chọn giải pháp hay chạy experiment. Capsule lưu có provenance, owner
  và điều kiện kiểm tra độ mới. Independent decision review và fresh direct
  document review: READY. Đọc lại parent H1 và H1-T1/H1-T11/H1-T12/H1-T13
  sau draft: NO CONFLICT. Chưa implementation; chưa commit tài liệu.
- **H1-T16 Explore** đã hoàn tất planning trong
  [implementation plan](../tasks/H1-T16-explore.md), implementation `ready`.
  User đã xác nhận mỗi lần Explore xử lý một choice question cụ thể, so sánh
  các option thực sự khác nhau không theo quota, và cân nhắc status quo khi
  viable. Tiêu chí lấy từ Intent, contract, Constitution và Current Truth;
  hard constraint tách khỏi preference, evidence tách khỏi assumption, không
  dùng score bắt buộc. Capsule conversational trả recommendation hoặc exact
  unresolved discriminator cùng provenance về Router; không approve, execute,
  hoặc tự gọi Research/Spike. Independent decision review sau khi bổ sung
  evidence/assumption provenance và direct-document review: READY. Đọc lại
  parent H1 và H1-T0/H1-T1/H1-T11/H1-T12/H1-T13 sau draft: NO CONFLICT.
  Chưa implementation; chưa commit tài liệu.
- Roadmap component H1-T2–H1-T10 cũ đã được
  [lưu để tham khảo](H1-archived-component-roadmap.md), không còn là task active.
  Backlog H1-T17–H1-T23 đi theo từng node của H1-T0; tên là placeholder,
  chưa phê duyệt implementation contract. H1-T24 ghi nhận workflow quản lý
  Constitution rule đã defer. Các điểm defer khác nằm trong
  [H1 deferred-work register](H1-deferred-work.md).
- Giữ **E8-S1: Connect the extension to repository state** ở trạng thái
  `needed`; chưa planning E8 cho đến khi H1 hoàn tất hoặc người dùng đổi ưu
  tiên rõ ràng.

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

E2-S5-T1, E2-S6-T1, E4-S1-T1, E4-S4-T1, E4-S5-T1, E5-S2-T1, E5-S3-T1, E5-S4-T1, E5-S5-T1, E4-S3-T1 và E1-S4-T1 đã implementation xong. E3-S2-T1 đã planning xong và sẵn sàng implementation.

**E3-S3-T1** đã planning xong nhưng implementation còn `blocked` bởi E3-S2-T1; E4-S3-T1 đã hoàn tất. Dependency cross-check đã đọc lại E3-S2/E3-S2-T1 và E4-S3/E4-S3-T1, kết luận `NO CONFLICT`.

**E6-S1-T1** đã implementation và verification xong với comment creation/listing, actor attribution, migration 0008, deterministic read-only ordering và linked-worktree concurrency coverage.

**E6-S2-T1** đã implementation và verification xong với hard delete theo actor được chọn, task-scoped lookup, stable ownership failures và transactional concurrency behavior.

**E6-S3-T1** đã planning xong và sẵn sàng implementation. Task này khóa comment immutability bằng regression coverage, không thêm edit command hay migration; correction là hai operation delete rồi add độc lập.

**E3-S4-T1** đã implementation và verification xong với atomic unused-status deletion, active/archived usage guard, order compaction và linked-worktree concurrency coverage. Dependency cross-check đã đọc lại E3-S2/E3-S2-T1 và E2-S1/E2-S1-T1, kết luận `NO CONFLICT`.

**E7-S1-T1** đã implementation và verification xong với global/idempotent `--json`, một response envelope, JSON-aware parse errors, stdout/stderr, exit taxonomy, non-interactive confirmation và partial-uninstall reporting; không đổi domain payload hay persistence.

**E7-S2-T1** đã implementation và verification xong với command/help inventory, actor-boundary checks và canonical-store lifecycle qua main/linked worktrees.

**E7-S3-T1** đã planning xong và sẵn sàng implementation. Task audit, chuẩn hóa help/wiring và regression-test command surface cho status, parent/dependency mutations, hierarchy và relationship maps; availability/blockers/claims thuộc E7-S4. Dependency cross-check đã đọc lại E3-S1–E3-S4, E4-S1–E4-S5, E4-S5-T2, E7-S1 và các implementation task trực tiếp, kết luận `NO CONFLICT`. Decision review và direct document review đều kết luận `READY`.

**E7-S4-T1** đã planning xong và sẵn sàng implementation. Task audit, chuẩn hóa help/wiring và regression-test năm leaf `task available/blockers/claim/claim-next/unclaim`, gồm hai invocation form owner/force của unclaim, actor placement, atomic-versus-informative distinction và E7-S1 transport; không thêm alias, payload, migration hay persistence. Dependency cross-check đã đọc lại E4-S3, E4-S4, E5-S1–E5-S5, E7-S1 và các implementation task trực tiếp, kết luận `NO CONFLICT`. Decision review và independent document review đều kết luận `READY`.

**E7-S5-T1** đã planning xong và sẵn sàng implementation. Task audit, chuẩn hóa help/wiring và regression-test exact `task comment add/list/delete` surface, actor placement, immutability guidance, active/archived parity và E7-S1 transport; không thêm alias, edit operation, alternate input, payload, migration hay persistence. Dependency cross-check đã đọc lại E6-S1–E6-S3, E7-S1 và các implementation task trực tiếp, kết luận `NO CONFLICT`. Decision review và direct document review đều kết luận `READY`.

**E9-S1-T1** đã implementation và verification xong với deterministic gated child-process races cho specified claim và claim-next qua main/linked worktrees; không đổi production behavior.

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
- Sau khi critical path hoàn tất, chốt E3-S2 với top-level `status list/create/rename/move`, custom statuses mới mặc định incomplete, code immutable và tên unique theo ASCII case-insensitive qua migration `0009`.
- Chốt create append hoặc đặt before/after; rename chỉ custom; reorder cả default/custom với contiguous zero-based order, atomic immediate transactions và valid no-write no-ops.
- Chốt logical-user-only authority, exact human/JSON shapes và stable status errors; decision review kết luận READY trước khi tạo tài liệu.
- Chốt E3-S3 với `status set-completed`, áp dụng cho cả default/custom status và định nghĩa completion ở cấp canonical repository, không phải per-task hoặc cross-repository global.
- Chốt deterministic impact gồm active tasks dùng status và qualifying direct downstream tasks, với availability, claims và unresolved upstreams trước/sau; archived và recursive-only tasks bị loại.
- Chốt explicit confirmation theo precedent E2-S6, valid no-write no-op, transactional impact recalculation, preserved claims và actual committed impact; decision review và direct document review đều kết luận READY.
- Đọc lại direct dependency E3-S2/E3-S2-T1 và E4-S3/E4-S3-T1; cross-check terminology, identity, persistence, output/error, concurrency và shared invariants kết luận `NO CONFLICT`. E3-S3-T1 bị block đến khi E3-S2-T1 hoàn tất.
- Chốt E3-S4 với `status delete <code>`, logical-user only, không confirmation; chỉ custom status không được active hoặc archived task nào dùng mới có thể bị xóa.
- Chốt success trả pre-delete status snapshot, order còn lại được compact atomically; lỗi `STATUS_IN_USE` trả `{code, taskCount}`, còn default immutability dùng human message trung lập cho rename/delete.
- Đọc lại E3-S2/E3-S2-T1 và E2-S1/E2-S1-T1; cross-check identity, task foreign key, ordering, output/error, migration và concurrency kết luận `NO CONFLICT`. Decision review và document review cuối đều kết luận READY.
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
- Chốt E6-S2 với `task comment delete <task-id> <comment-id>`, logical user được xóa mọi comment và declared agent chỉ được xóa comment mang đúng UUID của mình.
- Chốt actor selection kế thừa E6-S1: bỏ `--agent` chọn logical `user`; đây là trust convention chứ không xác thực physical caller, và authentication/credentials nằm ngoài scope.
- Chốt task-scoped `COMMENT_NOT_FOUND`, stable `COMMENT_DELETE_FORBIDDEN`, hard delete không confirmation, trả deleted E6-S1 comment shape, không đổi task metadata và serialize concurrent deletes trong immediate transaction.
- Viết epic/task E6-S2; decision review kết luận READY và task sau đó đã được implementation session hiện tại claim trên E6-S1-T1.
- Chốt E6-S3 là invariant hardening: supported core, CLI và UI consumer không được update comment in place; không thêm edit command, migration, trigger, revision metadata hoặc audit history.
- Chốt correction là hai operation E6-S2 delete rồi E6-S1 add độc lập, không atomic compensation; comment mới luôn có UUID mới và capture timestamp riêng nhưng timestamp value không bắt buộc khác comment cũ.
- Chốt focused architecture guard cho public core comment operations và production `UPDATE task_comments`, cùng CLI surface tests; E8-S6 UI phải kế thừa immutable core boundary.
- Phân loại acceptance impact E6-S3 là `add` cho combined correction journey `AT-E6-S3-001`; hai independent reviews sau chỉnh sửa đều kết luận READY.
- Chốt E7-S1 là output-boundary hardening trên contract E1-S1: một global/idempotent `--json`, một compact envelope trên stdout, stderr rỗng trong JSON mode, human errors trên stderr và giữ nguyên payload/error domain hiện hữu.
- Chốt exact argv token `--json` làm parse-error mode signal; unknown/missing/invalid arguments trả JSON exit `2`, typo không được suy đoán. Root/nested help và version luôn giữ Clap text cùng exit `0`, kể cả khi có `--json`.
- Chốt JSON và non-TTY không prompt; operation có confirmation dùng explicit `--yes` hoặc error exit `2`, còn human TTY cancellation là success exit `0`.
- Chốt partial uninstall chỉ phát một error envelope và chuyển toàn bộ `UninstallResult` hiện hữu vào `error.details`, giữ failure code, exit precedence, committed effects và recovery information.
- Viết epic/task E7-S1; decision review và direct document review đều kết luận READY. Đọc lại E1-S1/E1-S1-T1 và cross-check terminology, ownership boundary, output/error, confirmation, filesystem semantics cùng dependency type kết luận `NO CONFLICT`.
- Chốt E7-S4 giữ năm leaf `task available`, `blockers`, `claim`, `claim-next`, `unclaim`; owner và force-unclaim là hai option path trên cùng leaf, không thêm top-level group hoặc alias.
- Chốt actor chỉ xuất hiện bắt buộc trên claim, claim-next và owner-unclaim; availability/blockers không có actor, force-unclaim là logical-user path với confirmation hiện hữu.
- Chốt E7-S4 là audit/hardening story, kế thừa nguyên availability, conflict/empty, transaction, claim-status independence và E7-S1 transport; help-and-test-only implementation hợp lệ nếu audit không tìm thấy production gap. Dependency cross-check kết luận `NO CONFLICT`; independent review sau chỉnh sửa kết luận `READY`.
- Chốt E7-S5 giữ exact nested `task comment add/list/delete` surface, không có top-level alias, alternate content input hoặc edit-like operation; actor placement và ownership kế thừa E6-S1/E6-S2.
- Chốt comment-group help nêu immutable delete-then-add correction là hai operation độc lập; leaf help chỉ tập trung vào invocation, required values và actor selector.
- Chốt active/archived parity, comment shape, ordering, empty success, hard-delete result, errors và task invariants giữ nguyên theo E6-S1–S3; toàn bộ command kế thừa E7-S1 transport.
- Chốt E7-S5 là audit/hardening story; help-and-test-only implementation hợp lệ nếu không tìm thấy production gap. Viết epic/task, review quyết định và tài liệu đều kết luận READY; dependency cross-check E6-S1–S3/E7-S1 kết luận `NO CONFLICT`.

## Quy tắc planning cho mỗi story

### 1. Đọc source requirement

- Đọc trực tiếp section của story trong `docs/PRD.md`; không dựa chỉ vào handoff này.
- Đọc thêm các section PRD được story tham chiếu: domain rules, functional requirements, workflows, validation, NFR, MVP acceptance criteria và dependency stories.
- Đọc trực tiếp mọi epic contract trong `contract_depends_on` và implementation task tương ứng nếu đã tồn tại; không dựa vào riêng dashboard hoặc bản tóm tắt handoff.
- PRD là source of truth cho product behavior; epic/task ghi solution decisions.

### 2. Kiểm tra eligibility và claim planning

- `contract_depends_on` chỉ chứa story ID. `depends_on` trong technical task chỉ chứa implementation-task ID; không dùng lẫn hai loại.
- Một story đủ điều kiện planning khi mọi story trong toàn bộ transitive closure của `contract_depends_on` có planning `done` và epic contract đã tồn tại.
- Các story đủ điều kiện và không nằm trong dependency path của nhau có thể được planning song song, kể cả khi thuộc cùng epic.
- Trước khi phỏng vấn, claim story bằng cách đổi Planning từ `needed` sang `in_progress` trong `docs/STATUS.md`, rồi đọc lại dòng vừa cập nhật. Nếu session khác đã claim, dừng story đó và chọn story đủ điều kiện khác.
- Mỗi session chỉ sở hữu epic/task của story đã claim. Không sửa file story của session khác; khi cập nhật dashboard hoặc handoff phải giữ nguyên thay đổi đồng thời.
- Giữ claim `in_progress` cho đến khi tạo file, hoàn tất cross-check dependency, review và cập nhật handoff. Nếu planning dừng giữa chừng, không đánh dấu `done`.

### 3. Tóm tắt context trước khi planning

- Trước khi bắt đầu phỏng vấn hoặc đề xuất hướng implementation, tóm tắt lại context của story cho người dùng.
- Tóm tắt phải nêu ngắn gọn outcome, behavior bắt buộc từ PRD, dependencies, các quyết định đã kế thừa từ story trước và những điểm còn mở cần planning.
- Chỉ bắt đầu câu hỏi planning sau khi context này đã được trình bày, để người dùng có cùng baseline khi đưa ra quyết định.

### 4. Phỏng vấn người dùng

- Chọn một story `needed` đã đủ điều kiện theo contract graph; không bắt buộc theo thứ tự epic.
- Chỉ hỏi điểm còn mơ hồ hoặc có nhiều hướng implementation ảnh hưởng observable behavior.
- Đề xuất phương án mặc định rõ ràng để người dùng có thể trả lời ngắn.
- Giữ MVP đơn giản; không đào sâu adversarial filesystem behavior, recovery protocol hoặc edge case hiếm nếu PRD không yêu cầu.
- Khi người dùng chốt trade-off khác PRD, nêu conflict và xin phép cập nhật PRD trước khi ghi plan.
- Một session không bắt đầu story thứ hai trước khi story hiện tại đạt READY và tài liệu đã được tạo. Session khác có thể đồng thời planning story độc lập đã claim riêng.

### 5. Review sau mỗi vòng trả lời

Chỉ bắt đầu review khi người dùng đã xác nhận một hoặc nhiều quyết định planning. Không gọi review khi người dùng đang hỏi thêm về requirement, phản biện giả định, so sánh phương án, hoặc chưa thể hiện rằng họ đã chốt lựa chọn. Tiếp tục trao đổi tự nhiên cho đến khi có quyết định rõ ràng; nếu chưa chắc, hỏi lại thay vì suy diễn một câu hỏi thành approval.

Sau mỗi vòng quyết định đã được người dùng xác nhận, spawn một agent độc lập để đánh giá nội dung theo 5 tiêu chí:

1. Hướng implementation rõ ràng.
2. Có cách test.
3. Có cách verify đạt yêu cầu.
4. Có đủ reference cần thiết.
5. Có output và acceptance criteria đầy đủ, gồm functional và non-functional.

Agent review phải trả `READY` hoặc `NOT READY` và blocker cụ thể. Nếu `NOT READY`, tiếp tục phỏng vấn những blocker quan trọng. Không mở rộng sang chi tiết ít giá trị chỉ để đạt độ bao phủ lý thuyết.

Sau khi viết file và hoàn tất dependency cross-check bên dưới, yêu cầu agent review trực tiếp PRD/epic/task để tìm contradiction quan trọng. Sửa blocker, chạy lại cross-check bị ảnh hưởng, rồi review lại.

### 6. Tạo output

Khi READY, tạo:

```text
docs/epics/<STORY-ID>-<slug>.md
docs/tasks/<STORY-ID>-T1-<slug>.md
```

Epic plan nên gồm:

- Frontmatter `contract_depends_on` khớp chính xác contract dependency trong PRD và `docs/STATUS.md`.

- Outcome và user story.
- Scope/out of scope.
- Product/solution decisions.
- Functional acceptance criteria.
- Non-functional acceptance criteria.
- Test/verification approach.
- Link tới implementation task.

Technical task nên gồm:

- Frontmatter `depends_on` chỉ rõ các implementation task phải `done` trước khi task này có thể `ready`.

- Objective và deliverables.
- Proposed structure và technical choices.
- Implementation flow đủ để coding agent bắt đầu.
- Output/error contract cần thiết cho story.
- Test plan và verification commands.
- Definition of done.
- References.

Giữ plan đủ cụ thể để implement nhưng không biến thành line-by-line implementation hoặc catalogue mọi edge case.

### 7. Cross-check contract dependency sau khi tạo file

Đây là gate bắt buộc sau khi tạo epic/task và trước khi đánh dấu planning `done`:

1. Đọc lại phiên bản hiện tại của từng epic trong `contract_depends_on` và technical task tương ứng nếu có; không dùng nội dung đã nhớ từ đầu session.
2. So sánh contract mới với dependency về terminology, identity, ownership, state transitions, persistence boundary, output/error shape, concurrency và invariant được tái sử dụng.
3. Xác nhận story mới consume contract hiện có thay vì định nghĩa model hoặc source of truth cạnh tranh.
4. Kiểm tra `contract_depends_on` của epic khớp PRD và dashboard; kiểm tra `depends_on` của task đủ cho implementation ordering nhưng không đưa dependency chỉ mang tính tham khảo vào làm blocker.
5. Nếu dependency contract đã thay đổi trong lúc planning hoặc có contradiction, giữ planning `in_progress`, sửa plan hoặc quay lại phỏng vấn. Thay đổi observable behavior trong PRD vẫn cần người dùng chấp thuận.
6. Ghi kết quả cross-check ngắn trong phần review/handoff của story, gồm các dependency đã đọc và kết luận `NO CONFLICT` hoặc blocker còn lại.

Chỉ sau gate này và document review cuối mới đổi planning sang `done`, rồi tính implementation `ready`/`blocked` từ task-level `depends_on`.

### 8. Kiểm tra trước handoff

- Chạy `git diff --check`.
- Kiểm tra link giữa PRD, epic và task.
- Xác nhận PRD/epic/task không mâu thuẫn.
- Xác nhận dependency cross-check đã đọc lại tất cả direct contract dependencies và không còn conflict.
- Không sửa hoặc xóa unrelated user files.
- Mọi shell command phải prefix bằng `rtk` theo repository instruction.

## Trạng thái toàn bộ backlog

Xem dashboard tập trung tại [Planning and implementation status](../STATUS.md). Frontmatter trong từng epic/task document là nguồn trạng thái của document đó; dashboard bao gồm thêm các story vẫn cần planning.

## Cách tiếp tục

Nếu tiếp tục implementation:

1. Xem [status dashboard](../STATUS.md) để chọn task implementation `ready`; H1-T1 Constitution là ưu tiên H1 hiện tại. Claim task trước khi sửa source.

Nếu tiếp tục planning:

1. H1-T0 planning đã hoàn tất; H1-T1 Constitution planning đã hoàn tất và
   implementation task đang `ready`. Claim H1-T1 trước khi thay đổi source.
2. Chọn node tiếp theo từ flow H1-T0 và plan từ nhu cầu thực của node đó.
   Xác định dependency khi planning, không dùng roadmap H1-T2–H1-T10 cũ hoặc
   pre-author shared schema/component cho toàn bộ harness. Giữ các điểm defer
   trong register cho đến khi được planning riêng.
3. Giữ toàn bộ story E8, bao gồm E8-S1, ở trạng thái `needed` cho đến khi H1
   hoàn tất hoặc người dùng thay đổi ưu tiên rõ ràng.
4. Khi quay lại E8, re-plan toàn bộ epic trước khi claim một story riêng lẻ;
   xác định user journeys, VS Code surfaces, navigation, interaction/state
   model, cross-cutting UI requirements và technical foundation dùng chung.
