# GitHub CLI Setup for Remote Working

## Current Configuration

The GitHub CLI (gh) is properly configured for remote working with this repository.

### Authentication Status
- **Logged in account**: syt942
- **Authentication method**: OAuth token
- **Git operations protocol**: HTTPS
- **Token scopes**: gist, read:org, repo, workflow

### Repository Access
- **Repository**: Cicdsd/Proxy-desktop-browser
- **Organization**: Cicdsd
- **Permissions**: admin, maintain, pull, push, triage (full access)

## Verified Capabilities

The following operations have been verified to work:

1. Git Push - Successfully push commits to remote branches
2. Issue Management - Can list, create, and manage issues
3. Pull Request Management - Can list, create, and manage PRs
4. Repository API Access - Full API access to repository

## Common Commands

### Authentication
- Check auth status: gh auth status
- Re-authenticate if needed: gh auth login

### Repository Operations
- View repository info: gh repo view
- Clone repository: gh repo clone Cicdsd/Proxy-desktop-browser

### Issue Management
- List issues: gh issue list
- Create new issue: gh issue create --title "Title" --body "Description"
- View specific issue: gh issue view <issue-number>

### Pull Request Management
- List PRs: gh pr list
- Create PR for current branch: gh pr create --title "Title" --body "Description"
- View PR status: gh pr status
- Checkout a PR locally: gh pr checkout <pr-number>

### Workflow Management
- List workflows: gh workflow list
- View workflow runs: gh run list
- View specific run: gh run view <run-id>

## Troubleshooting

If you encounter authentication issues:
1. Check token expiration: gh auth status
2. Refresh authentication: gh auth refresh
3. Re-login if needed: gh auth login

## Configuration Files
- gh config: /root/.config/gh/hosts.yml
- git config: .git/config (in repository root)
