{{/*
Expand the name of the chart.
*/}}
{{- define "klask.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
We truncate at 63 chars because some Kubernetes name fields are limited to this (by the DNS naming spec).
If release name contains chart name it will be used as a full name.
*/}}
{{- define "klask.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- $name := default .Chart.Name .Values.nameOverride }}
{{- if contains $name .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}
{{- end }}

{{/*
Create chart name and version as used by the chart label.
*/}}
{{- define "klask.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "klask.labels" -}}
helm.sh/chart: {{ include "klask.chart" . }}
{{ include "klask.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{/*
Selector labels
*/}}
{{- define "klask.selectorLabels" -}}
app.kubernetes.io/name: {{ include "klask.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Create the name of the service account to use
*/}}
{{- define "klask.serviceAccountName" -}}
{{- if .Values.serviceAccount.create }}
{{- default (include "klask.fullname" .) .Values.serviceAccount.name }}
{{- else }}
{{- default "default" .Values.serviceAccount.name }}
{{- end }}
{{- end }}

{{/*
PostgreSQL service name
*/}}
{{- define "klask.postgresql.servicename" -}}
{{- if .Values.postgresql.enabled -}}
{{- printf "%s-postgresql" (include "klask.fullname" .) -}}
{{- else -}}
{{- .Values.postgresql.external.host -}}
{{- end -}}
{{- end -}}


{{/*
PostgreSQL secret name
*/}}
{{- define "klask.postgresql.secretName" -}}
{{- if .Values.postgresql.auth.existingSecret -}}
{{- .Values.postgresql.auth.existingSecret -}}
{{- else -}}
{{- printf "%s-postgresql" (include "klask.fullname" .) -}}
{{- end -}}
{{- end -}}

{{/*
Backend secret name
*/}}
{{- define "klask.backend.secretName" -}}
{{- if .Values.backend.existingSecret -}}
{{- .Values.backend.existingSecret -}}
{{- else -}}
{{- printf "%s-backend" (include "klask.fullname" .) -}}
{{- end -}}
{{- end -}}

{{/*
PostgreSQL fullname
*/}}
{{- define "klask.postgresql.fullname" -}}
{{- printf "%s-postgresql" (include "klask.fullname" .) -}}
{{- end -}}

{{/*
PostgreSQL host - returns service name if embedded, external host otherwise
*/}}
{{- define "klask.postgresql.host" -}}
{{- if .Values.postgresql.enabled -}}
{{- include "klask.postgresql.fullname" . -}}
{{- else -}}
{{- required "postgresql.external.host is required when postgresql.enabled is false" .Values.postgresql.external.host -}}
{{- end -}}
{{- end -}}

{{/*
PostgreSQL port
*/}}
{{- define "klask.postgresql.port" -}}
{{- if .Values.postgresql.enabled -}}
5432
{{- else -}}
{{- .Values.postgresql.external.port | default 5432 -}}
{{- end -}}
{{- end -}}

{{/*
PostgreSQL database name
*/}}
{{- define "klask.postgresql.database" -}}
{{- if .Values.postgresql.enabled -}}
{{- .Values.postgresql.auth.database -}}
{{- else -}}
{{- required "postgresql.external.database is required when postgresql.enabled is false" .Values.postgresql.external.database -}}
{{- end -}}
{{- end -}}

{{/*
PostgreSQL username
*/}}
{{- define "klask.postgresql.username" -}}
{{- if .Values.postgresql.enabled -}}
{{- .Values.postgresql.auth.username -}}
{{- else -}}
{{- required "postgresql.external.username is required when postgresql.enabled is false" .Values.postgresql.external.username -}}
{{- end -}}
{{- end -}}

{{/*
PostgreSQL password (for URL construction)
*/}}
{{- define "klask.postgresql.password" -}}
{{- if .Values.postgresql.enabled -}}
{{- .Values.postgresql.auth.password | default (randAlphaNum 32) -}}
{{- else -}}
{{- required "postgresql.external.password is required when postgresql.enabled is false" .Values.postgresql.external.password -}}
{{- end -}}
{{- end -}}

{{/*
Construct PostgreSQL connection URL
*/}}
{{- define "klask.postgresql.databaseUrl" -}}
{{- if and (not .Values.postgresql.enabled) .Values.postgresql.external.url -}}
{{- .Values.postgresql.external.url -}}
{{- else -}}
{{- $host := include "klask.postgresql.host" . -}}
{{- $port := include "klask.postgresql.port" . -}}
{{- $database := include "klask.postgresql.database" . -}}
{{- $username := include "klask.postgresql.username" . -}}
{{- $password := include "klask.postgresql.password" . -}}
{{- printf "postgresql://%s:%s@%s:%s/%s" $username $password $host $port $database -}}
{{- end -}}
{{- end -}}

{{/*
PostgreSQL labels
*/}}
{{- define "klask.postgresql.labels" -}}
{{ include "klask.labels" . }}
app.kubernetes.io/component: postgresql
{{- end -}}

{{/*
PostgreSQL selector labels
*/}}
{{- define "klask.postgresql.selectorLabels" -}}
{{ include "klask.selectorLabels" . }}
app.kubernetes.io/component: postgresql
{{- end -}}