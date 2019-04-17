# {{title}}

Last updated: {{timestamp}}

{% for entry in entries %}
  {{loop.index}}. {{entry.title}} - [EPUB]({{entry.path | urlencode}}) | [Website]({{entry.url}}) | [Repository]({{entry.repo_url}})
  Commit: {{entry.commit_sha}} ({{entry.last_modified}})
{% endfor %}
