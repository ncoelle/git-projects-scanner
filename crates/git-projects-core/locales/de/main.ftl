# Git Projects Scanner - Deutsche Übersetzungen

# Allgemeine Nachrichten
app-name = Git-Projekte-Scanner
app-description = Git-Repositories im lokalen Dateisystem scannen und katalogisieren

# Scan-Nachrichten
scan-started = Suche nach Git-Repositories...
scan-started-path = Durchsuche: { $path }
scan-progress = Bisher { $count } { $count ->
    [one] Repository
    *[other] Repositories
} gefunden...
scan-complete = Scan abgeschlossen! { $count } { $count ->
    [one] Repository
    *[other] Repositories
} gefunden.
scan-no-results = Keine Git-Repositories gefunden.

# Tabellenkopfzeilen
header-name = Name
header-path = Pfad
header-remotes = Remotes
header-config = Konfiguration
header-submodule = Submodul
header-has-submodules = Hat Submodule
header-last-scanned = Zuletzt gescannt
header-service = Dienst
header-account = Account

# Remote-Informationen
remote-none = (keine)
remote-count = { $count } { $count ->
    [one] Remote
    *[other] Remotes
}

# Konfigurations-Informationen
config-local = Lokal
config-global = Global
config-system = System
config-user = { $name } <{ $email }>
config-name-only = { $name }
config-email-only = <{ $email }>
config-none = (nicht konfiguriert)

# Submodul-Status
submodule-yes = Ja
submodule-no = Nein

# Sortierprofile
sort-name = Nach Name (alphabetisch)
sort-path = Nach Pfad (alphabetisch)
sort-recent = Nach letztem Scan (neueste zuerst)
sort-service = Nach Dienst (gruppiert)

# Ausgabeformate
output-table = Tabellenformat (lesbar)
output-json = JSON-Format (maschinenlesbar)

# Fehlermeldungen
error-io = E/A-Fehler: { $details }
error-git-open = Git-Repository konnte nicht geöffnet werden: { $path }
error-git-config = Git-Konfiguration konnte nicht gelesen werden: { $path }
error-path-not-found = Pfad existiert nicht: { $path }
error-not-directory = Pfad ist kein Verzeichnis: { $path }
error-invalid-locale = Ungültige Locale: { $locale }
error-localization = Lokalisierungsfehler: { $details }
error-json = JSON-Fehler: { $details }
error-unknown = Ein Fehler ist aufgetreten: { $details }

# CLI-Hilfetext
help-root = Wurzelverzeichnis zum Scannen (kann mehrfach angegeben werden)
help-depth = Maximale Rekursionstiefe (Standard: 3)
help-no-symlinks = Symbolischen Links nicht folgen
help-no-submodules = Submodul-Repositories nicht einbeziehen
help-sort = Sortierprofil: name, path, recent oder service
help-json = Ausgabe als JSON statt Tabelle
help-verbose = Detaillierten Scan-Fortschritt anzeigen
help-locale = Locale für Nachrichten (z.B. en, de)

# Ausführliche Ausgabe
verbose-analyzing = Analysiere: { $path }
verbose-found-repo = Repository gefunden: { $name }
verbose-found-submodule = Submodul gefunden: { $name }
verbose-skipping = Überspringe: { $path }
verbose-warning = Warnung: { $message }

# Statusmeldungen
status-ok = ✓ OK
status-warning = ⚠ Warnung
status-error = ✗ Fehler