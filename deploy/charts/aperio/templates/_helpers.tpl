{{/*
Expand the name of the chart.
*/}}
{{- define "aperio.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
*/}}
{{- define "aperio.fullname" -}}
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
{{- define "aperio.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "aperio.labels" -}}
helm.sh/chart: {{ include "aperio.chart" . }}
{{ include "aperio.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{/*
Selector labels
*/}}
{{- define "aperio.selectorLabels" -}}
app.kubernetes.io/name: {{ include "aperio.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Sync selector labels
*/}}
{{- define "aperio.syncSelectorLabels" -}}
app.kubernetes.io/name: {{ include "aperio.name" . }}-sync
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Create the name of the service account to use
*/}}
{{- define "aperio.serviceAccountName" -}}
{{- if .Values.serviceAccount.create }}
{{- default (include "aperio.fullname" .) .Values.serviceAccount.name }}
{{- else }}
{{- default "default" .Values.serviceAccount.name }}
{{- end }}
{{- end }}

{{/*
Image reference
*/}}
{{- define "aperio.image" -}}
{{- $tag := default .Chart.AppVersion .Values.image.tag -}}
{{- printf "%s:%s" .Values.image.repository $tag -}}
{{- end }}

{{/*
Sync image reference
*/}}
{{- define "aperio.syncImage" -}}
{{- printf "%s:%s" .Values.syncImage.repository .Values.syncImage.tag -}}
{{- end }}

{{/*
PVC name
*/}}
{{- define "aperio.pvcName" -}}
{{- printf "%s-vault" (include "aperio.fullname" .) -}}
{{- end }}

{{/*
ConfigMap name
*/}}
{{- define "aperio.configMapName" -}}
{{- printf "%s-config" (include "aperio.fullname" .) -}}
{{- end }}

{{/*
Secret name
*/}}
{{- define "aperio.secretName" -}}
{{- printf "%s-secret" (include "aperio.fullname" .) -}}
{{- end }}
