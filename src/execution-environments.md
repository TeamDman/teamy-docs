# Locate work across execution environments

A conversation can coordinate work in several places. Record where an operation runs, which identity and policy apply, and where its output lives. The same path spelling in two environments is not evidence of a shared filesystem. This conceptual model follows the public documentation; it does not claim an internal schema.

| Layer | What it identifies | Evidence and boundary |
| --- | --- | --- |
| Conversation and agent | Instructions, context and task coordination | [ChatGPT Work and Codex](https://help.openai.com/en/articles/20001275-chatgpt-work-and-codex) describes separate Codex and ChatGPT histories. A task delegated elsewhere has its own execution context. |
| Dot cloud computer | A dot's cloud execution and browser context | [Dot documentation](https://learn.chatgpt.com/docs/dots) describes its cloud computer, separate browser sessions and delegation. A browser login does not automatically transfer to a connected computer or Codex task. |
| Connected user computer | Commands and files on an authorized local host | [Local computer access](https://learn.chatgpt.com/docs/enterprise/cloud-local-access) describes delegated local execution and its access controls. The host and its policies are distinct from the dot's cloud computer. |
| Cloud environment definition and configuration | Reusable setup, repositories, tools and access settings | [Cloud environments](https://learn.chatgpt.com/docs/environments/cloud-environments) describes preparing and testing an environment. Do not infer an undocumented internal representation or first-creation sequence. |
| Published prepared filesystem | The saved setup from which new tasks start | The same documentation describes publishing or republishing changes to the prepared filesystem. Check the applicable access settings separately; do not assume every setting has the same lifecycle as filesystem changes. |
| Task workspace or instance | An isolated environment in which a task executes | New tasks start from the published prepared filesystem; existing tasks retain their own state. Persist important work in source control or a suitable artifact store. |
| Space, Page or saved file | A content identity, distinct from a process filesystem or conversation | [Space agents](https://learn.chatgpt.com/docs/space/agents) describes shared content. [File retention](https://help.openai.com/en/articles/8983778-chat-and-file-retention-in-chatgpt) says saved files can be separate from a conversation; it also uses the name Library for some accounts. Deleting a conversation does not necessarily delete a separately saved file. |

Availability varies by account, workspace, settings and time. [Dot security](https://learn.chatgpt.com/docs/enterprise/dots-security) describes separate cloud-computer controls and policies governing delegated tasks. A capability exposed to one agent or browser is not proof that another task can use it. A local checkout, a Git worktree, a cloud environment definition and a cloud task workspace are different objects; clarify which one a restriction addresses.

## Move a file deliberately

Identify the source instance, destination instance, permitted data and expected output. Choose an available route: an authorized connector or Space file, a repository for appropriate source, or a controlled direct transfer. Confirm actual availability and size limits. A failed upload or download is a failed route, not proof that all routes are unavailable. A file saved in a Space is a separate object; confirm its identity and version before retrieving it.

For a direct route, direction of the connection is independent of direction of the data. A reachable desktop receiver can accept a cloud upload; a reachable desktop server can supply a cloud download. Verify reachability separately in each direction and for the specific instance, protocol and policy. A previous successful connection in one direction proves neither the reverse direction nor access to a different environment. Limit the served files, authenticate peers, check content size and digest, handle existing destinations and clean up the temporary service. Do not publish private paths or credentials in plans or logs.

Teamy Mover has separate planning and transfer work. Verify the transfer modes and their constraints in the exact build being used; a proposed download mode or a prior experiment does not establish availability in every build. See [file movement](moving-files.md) and [capability planning](decision-planning.md).
