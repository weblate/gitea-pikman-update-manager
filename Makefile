export PATH := "$(PWD):$(PATH)"

all:
	true

install:
	cargo fetch
	cargo build --release
	mkdir -p $(DESTDIR)/usr/bin/
	mkdir -p $(DESTDIR)/usr/lib/pika/pikman-update-manager/scripts/
	mkdir -p $(DESTDIR)/usr/share/glib-2.0/schemas/
	mkdir -p $(DESTDIR)/usr/share/applications
	mkdir -p $(DESTDIR)/usr/share/icons/hicolor/scalable/apps
	cp -vf target/debug/pikman-update-manager $(DESTDIR)/usr/bin/
	cp -vf data/flatpak-installer $(DESTDIR)/usr/bin/
	cp -vf data/software-properties-gtk $(DESTDIR)/usr/bin/
	cp -vf target/debug/apt_update $(DESTDIR)/usr/lib/pika/pikman-update-manager/scripts/
	cp -vf target/debug/apt_full_upgrade $(DESTDIR)/usr/lib/pika/pikman-update-manager/scripts/
	cp -vf data/modify_repo.sh $(DESTDIR)/usr/lib/pika/pikman-update-manager/scripts/
	cp -vf data/*.gschema.xml $(DESTDIR)/usr/share/glib-2.0/schemas/
	cp -vf data/com.github.pikaos-linux.pikmanupdatemanager.svg $(DESTDIR)/usr/share/icons/hicolor/scalable/apps/
	cp -vf data/*.desktop $(DESTDIR)/usr/share/applications/
	cp -vfr data/polkit-1 $(DESTDIR)/usr/share/
	chmod 755 $(DESTDIR)/usr/bin/pikman-update-manager
	chmod 755 $(DESTDIR)/usr/bin/flatpak-installer
	chmod 755 $(DESTDIR)/usr/bin/software-properties-gtk
	chmod 755 $(DESTDIR)/usr/lib/pika/pikman-update-manager/scripts/*

install_no_build_debug:
	mkdir -p $(DESTDIR)/usr/bin/
	mkdir -p $(DESTDIR)/usr/lib/pika/pikman-update-manager/scripts/
	mkdir -p $(DESTDIR)/usr/share/glib-2.0/schemas/
	mkdir -p $(DESTDIR)/usr/share/applications
	mkdir -p $(DESTDIR)/usr/share/icons/hicolor/scalable/apps
	cp -vf target/debug/pikman-update-manager $(DESTDIR)/usr/bin/
	cp -vf data/flatpak-installer $(DESTDIR)/usr/bin/
	cp -vf data/software-properties-gtk $(DESTDIR)/usr/bin/
	cp -vf target/debug/apt_update $(DESTDIR)/usr/lib/pika/pikman-update-manager/scripts/
	cp -vf target/debug/apt_full_upgrade $(DESTDIR)/usr/lib/pika/pikman-update-manager/scripts/
	cp -vf data/modify_repo.sh $(DESTDIR)/usr/lib/pika/pikman-update-manager/scripts/
	cp -vf data/*.gschema.xml $(DESTDIR)/usr/share/glib-2.0/schemas/
	cp -vf data/com.github.pikaos-linux.pikmanupdatemanager.svg $(DESTDIR)/usr/share/icons/hicolor/scalable/apps/
	cp -vf data/*.desktop $(DESTDIR)/usr/share/applications/
	cp -vfr data/polkit-1 $(DESTDIR)/usr/share/
	chmod 755 $(DESTDIR)/usr/bin/pikman-update-manager
	chmod 755 $(DESTDIR)/usr/bin/flatpak-installer
	chmod 755 $(DESTDIR)/usr/bin/software-properties-gtk
	chmod 755 $(DESTDIR)/usr/lib/pika/pikman-update-manager/scripts/*