SHELL := /bin/sh

TF ?= terraform
TFLINT ?= tflint

TF_DOCKER_IMAGE ?= hashicorp/terraform:latest
TFLINT_DOCKER_IMAGE ?= ghcr.io/terraform-linters/tflint:latest

TF_DOCKER_RUN = docker run --rm -v "$(CURDIR):/workspace" -w /workspace -e TF_IN_AUTOMATION -e TF_VAR_repository_full_name -e TF_VAR_repository_token -e TF_VAR_neon_api_token -e TF_VAR_database_project_name -e TF_VAR_database_region_id -e TF_VAR_database_name -e TF_VAR_database_role_name -e TF_VAR_app_name -e TF_VAR_app_service_name -e TF_VAR_app_source_repository -e TF_VAR_app_source_branch -e TF_VAR_deployment_region -e TF_VAR_deployment_plan -e TF_VAR_app_root_directory -e TF_VAR_app_runtime -e TF_VAR_app_build_command -e TF_VAR_app_start_command -e TF_VAR_render_owner_id -e TF_VAR_health_monitor_name -e TF_VAR_app_healthcheck_url -e RENDER_API_KEY -e BETTERUPTIME_API_TOKEN $(TF_DOCKER_IMAGE)
TFLINT_DOCKER_RUN = docker run --rm -v "$(CURDIR):/workspace" -w /workspace $(TFLINT_DOCKER_IMAGE)

.PHONY: help fmt fmt-docker lint lint-docker init init-docker validate validate-docker plan plan-docker show-plan show-plan-docker apply apply-docker ci apply-ci

help:
	@echo "Native targets: fmt lint init validate plan show-plan apply ci apply-ci"
	@echo "Docker targets: fmt-docker lint-docker init-docker validate-docker plan-docker show-plan-docker apply-docker"

fmt:
	$(TF) fmt -check -recursive

fmt-docker:
	$(TF_DOCKER_RUN) fmt -check -recursive

lint:
	$(TFLINT) --init
	$(TFLINT) --recursive --format compact

lint-docker:
	$(TFLINT_DOCKER_RUN) --init
	$(TFLINT_DOCKER_RUN) --recursive --format compact

init:
	$(TF) init

init-docker:
	$(TF_DOCKER_RUN) init

validate:
	$(TF) validate

validate-docker:
	$(TF_DOCKER_RUN) validate

plan:
	$(TF) plan -out=tfplan

plan-docker:
	$(TF_DOCKER_RUN) plan -out=tfplan

show-plan:
	$(TF) show tfplan > plan.txt

show-plan-docker:
	$(TF_DOCKER_RUN) show tfplan > plan.txt

apply:
	$(TF) apply -auto-approve

apply-docker:
	$(TF_DOCKER_RUN) apply -auto-approve

ci: lint init plan show-plan

apply-ci: init apply
