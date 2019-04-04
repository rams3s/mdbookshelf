# The ebooks

Last updated: {{timestamp}}

{% for entry in entries %}
  {{loop.index}}. [{{entry.name}}]({{entry.path}}) - [Official]({{entry.url}}) - [Repository]({{entry.repo_url}}) - Last commit date: {{entry.last_modified}}
{% endfor %}
