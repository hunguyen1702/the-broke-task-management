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
- **E3-S1: Use default statuses**
  - [Epic plan](../epics/E3-S1-use-default-statuses.md)
  - [Implementation task](../tasks/E3-S1-T1-implement-default-status-codes.md)

Story tiếp theo:

- **E2-S5: Archive a task** — chốt archive eligibility, relationship/claim handling và observable output/errors.

### Implementation readiness tại thời điểm handoff

- **E1-S1-T1**, **E1-S2-T1**, **E1-S5-T1**, **E1-S3-T1**, **E3-S1-T1** và **E2-S1-T1** đã implementation xong; xem commit và dependency hiện hành trong [status dashboard](../STATUS.md).
- **E1-S4-T1** mới ở trạng thái planned và đang bị block đến khi E2-S1, E3-S1, E4-S2 và E5-S1 cung cấp task/status/claim models có thẩm quyền.
- **E3-S1-T1** đã hoàn tất tại commit `4d8ebb2`, cung cấp stable status codes và mở khóa task creation.
- **E2-S1-T1** đã hoàn tất tại commit `ca25528`, cung cấp task creation với explicit actor, atomic aggregate insert, stable task ID, status machine code và structured coding context.

Implementation tiếp theo có thể bắt đầu theo dashboard; **E2-S4-T1** đã ready, còn **E2-S3-T1** bị block bởi **E4-S1-T1**. Planning tiếp theo là **E2-S5: Archive a task**.

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

1. [E2-S4-T1: Implement structured task-context updates](../tasks/E2-S4-T1-implement-structured-task-context-updates.md) đã ready trên E2-S1-T1 và E2-S2-T1.
2. [E2-S3-T1: Implement task-content updates](../tasks/E2-S3-T1-implement-task-content-updates.md) chưa thể bắt đầu cho đến khi E4-S1 được planned và E4-S1-T1 cung cấp authoritative hierarchy model.
3. Chọn task `ready` từ [status dashboard](../STATUS.md), hoặc tiếp tục planning E4-S1 nếu muốn mở khóa E2-S3-T1.

Nếu tiếp tục planning:

1. Mở `docs/PRD.md` và tìm `Story E2-S5: Archive a task`.
2. Đọc lifecycle/domain rules và các E2, E4, E5 dependency contracts liên quan.
3. Tóm tắt context E2-S5, rồi hỏi người dùng các quyết định implementation còn thiếu, đặc biệt archive eligibility, active-claim handling, relationship preservation và idempotency/output.
4. Sau mỗi câu trả lời, spawn review agent theo 5 tiêu chí.
5. Khi READY, tạo epic và technical task E2-S5, review file, cập nhật danh sách trong handoff này.
