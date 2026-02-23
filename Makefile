.PHONY: build-backend build-front check-backend run-backend dev-front clean help

help:
	@echo "Available commands:"
	@echo "  make build         - Build both backend and frontend"
	@echo "  make build-backend - Build the backend"
	@echo "  make build-front   - Build the frontend"
	@echo "  make check-backend - Run cargo check on the backend"
	@echo "  make run-backend   - Run the backend"
	@echo "  make dev-front     - Run frontend in development mode"
	@echo "  make clean         - Clean both backend and frontend"

build: build-backend build-front

build-backend:
	$(MAKE) -C backend build

build-front:
	$(MAKE) -C front build

check-backend:
	$(MAKE) -C backend check

run-backend:
	$(MAKE) -C backend run

dev-front:
	$(MAKE) -C front dev

clean:
	$(MAKE) -C backend clean
	$(MAKE) -C front clean
