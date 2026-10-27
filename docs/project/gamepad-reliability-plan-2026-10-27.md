# Connexion de manette — plan final et réalisation du 27 octobre 2026

Référence étudiée : `96b89d8` (0.17.9). Ce document remplace la proposition initiale après
la [revue indépendante Claude](gamepad-reliability-claude-review-2026-10-27.md).
Le parcours de référence reste [pair-a-gamepad.md](../robot/pair-a-gamepad.md) ;
[udp-pad.md](../robot/udp-pad.md) possède la sélection de source.

## Périmètre retenu

Deux lots : statut de connexion honnête, puis `pad connect <MAC>`. On conserve configd
propriétaire de BlueZ, padd lecteur non privilégié et netpad.enabled sélecteur explicite.
Le mode de pilotage, la policy et l’activité des boutons restent hors de ce chantier.
Pas de nouveau daemon, d’oubli automatique, de bascule implicite de source ni de modification
du comportement moteur. L’implémentation est locale ; rien n’a été déployé sur le robot.

## Ce que l’incident établit

Dans la session [Microduck · Connexion au contrôleur et mises à jour](codex://threads/01a177de-4ce2-7022-8c2b-00f246bc8983),
le tour `01a17c28-a459-7b21-90fc-63e77100b49d` contient la sortie brute suivante (extrait,
sans clé de liaison) :

```text
pad     Pro Controller 98:B6:E9:64:FD:06  paired, not connected — switch the pad on
pad     Pro Controller 98:B6:E9:A4:FC:67  paired, not connected — switch the pad on
padd    active — driving whatever pad connects
2026-10-27T07:01:05.565487Z INFO configd: pairing a gamepad mac=None timeout=20s
2026-10-27T07:01:05.577641Z INFO configd::bluez: looking for a gamepad in pairing mode timeout=20s
2026-10-27T07:01:55.606403Z INFO configd::bluez: connect did not answer in time; asking to pair
2026-10-27T07:01:55.609789Z INFO configd::bluez: still not bonded paired=false connected=false
2026-10-27T07:01:55.615568Z WARN configd::bluez: pairing failed mac=98:B6:ED:A4:FC:67 reason=Rejected detail=org.freedesktop.DBus.Error.UnknownObject: Method "Pair" with signature "" on interface "org.bluez.Device1" doesn't exist
```

**Faits vérifiés dans le code :** les candidats non appairés gagnent sur les associations
existantes ; `one_face_per_pad` ne compare que les non-appairés. ED a pris le chemin
Connect-first, donc son snapshot n’avait pas Class. La découverte était déjà arrêtée avant
Connect. Après ses 30 secondes de timeout, Pair a visé un objet qui n’existait plus.
UnknownObject était classé Rejected, d’où un conseil inapproprié sur Privacy et un reboot.
Relancer Pair sur une association existante ne faisait que réaffirmer Trusted, sans Connect.

**Observations rapportées :** après oubli E9 et nouvel appairage, une connexion explicite a
encore été nécessaire. Le relais USB Mac a été arrêté ; netpad.enabled était finalement false,
padd actif et netpadd inactif. Le robot était graphite-boom, dernière adresse `192.168.1.82`.
Aucune lecture actuelle du robot n’est revendiquée par ce document.

**Hypothèses :** ED pourrait être une annonce LE temporaire du clone, comme le précédent
matériel documenté dans bluez.rs pour une autre unité, mais l’identité ED/E9 de cet incident
n’est pas prouvée. Les suffixes égaux ne suffisent pas. Le délai total de 50 s suggère un
candidat par nom seul, sauf arrivée classifiée dans la dernière seconde. L’éviction d’ED par
BlueZ après un délai est à mesurer : TemporaryTimeout et InterfacesRemoved manquent.
Le succès après oubli ne prouve pas des clés périmées ; E9 a aussi pu redevenir le candidat
non appairé préféré. L’association E9:64:FD:06 reste indépendante.

## Lot 1 — statut précis et messages corrigés

Fichiers : `configd/src/{pad,bluez,pad_hid,main}.rs`, `duck-ipc-proto/src/lib.rs`,
`robotctl/src/main.rs` et documentation du parcours.

- Le statut distingue adaptateur absent, éteint, allumé, inconnu et BlueZ indisponible.
  La lecture ne met pas l’adaptateur sous tension. Le premier adaptateur par chemin lexical
  est utilisé, comme pour l’appairage ; son chemin est affiché.
- Les associations du même adaptateur restent visibles même hors heuristique gamepad,
  ce qui couvre les appareils appairés explicitement par MAC.
- Chaque association porte paired, trusted, connected, ServicesResolved et une indication
  HID indépendante. La correspondance sysfs utilise l’adresse complète de `uniq`, sans tenir
  compte de la casse, sur un nœud d’entrée Bluetooth. Une entrée USB ne confirme jamais le
  Bluetooth. Un manque d’observabilité donne inconnu ; aucun suffixe n’est utilisé.
- La source configurée et les deux services padd/netpadd sont affichés séparément. Conflit
  ou divergence ne sont pas masqués. Une erreur BlueZ laisse accessibles les faits indépendants.
  Le backend indisponible ne se fait plus passer pour une liste vide de manettes.
- Un lien Connected ne produit plus « padd is driving ». Pair ne promet plus une reconnexion
  automatique. UnknownObject indique un appareil disparu ; il ne conseille ni Privacy ni reboot.
- La vérification finale d’une association ne construit plus de Pad avec paired/trusted vrais
  si l’objet a disparu. Elle renvoie un échec de vérification et demande de consulter le statut.

Critères : distinguer lien connecté sans nœud correspondant ; radio tardive sans conseil de
réappairage ; statut partiel utile ; aucun message affirmant une prise de contrôle.
Le statut ne prouve pas que padd a ouvert le nœud ni que les boutons sont utilisables.

## Lot 2 — `pad connect <MAC>`

Fichiers : backend configd et helper de reconnexion dans pad.rs, protocole IPC, CLI,
routage btd et tables explicites des autres services.

- La MAC est obligatoire et exacte. Aucun scan, choix implicite, appairage, suppression de
  clés, agent de pairing, reset de radio ni activation d’adaptateur éteint.
- Un appareil absent/non appairé sur l’adaptateur choisi est refusé. Une association existante
  est conservée ; la confiance manquante est réparée avant Connect.
- Pair et Connect prennent le même mutex. Le budget total de 15 s inclut l’attente du mutex,
  la lecture BlueZ, la confiance, Connect et la vérification finale.
- AlreadyConnected et InProgress sont observés, sans lancer un second Connect. La réussite
  nécessite une relecture paired/trusted/connected ; l’apparition HID reste une autre preuve.
- L’échec porte sa phase, le détail et le dernier état observé s’il existe. Le CLI échoue aussi
  avec `--json` quand le résultat de Pair ou Connect est un refus.
- `pad.connect` est une opération mutante de configd, donc soumise à may_mutate. Elle est
  disponible par BLE authentifié comme les autres mutations pad. WebRTC reste refusé en v1 ;
  l’exposition d’un passage de main à distance est une décision distincte.

Critères : connexion exacte sans effacer d’association ; idempotence ; pas d’attente infinie ;
aucun succès inventé après disparition ; autre association intacte ; permissions respectées.

## Vérifications locales

Tests ajoutés : séquence confiance/connexion/vérification, idempotence, MAC absente ou non
appairée, opération en cours bornée, Connect sans réponse, disparition après Connect,
conservation d’une autre association, statut partiel et configuration illisible, autorisation
avant mutation, rendus sans affirmation de pilotage et identité HID exacte (USB exclu).
Les tables de routage et de refus du workspace incluent explicitement la nouvelle méthode.

Le bilan final d’exécution des commandes de validation est consigné en fin de ce document.
Les tests ne nécessitent aucun robot. La découverte, les clés et l’ordre radio d’appairage
n’ont pas été changés par cette réalisation.

## Essais physiques ultérieurs, ensemble

Robot dans un état stable, aucune commande motrice envoyée par le diagnostic, protection de
prise en main conservée. Ne pas utiliser Pair comme commande de reconnexion : sur une carte
avec marqueur weird-ble, son reset radio peut déclencher la réaction native de perte de pad.

1. Lire status, les états des services, bluetoothctl info et les identités sysfs. Relever la
   version BlueZ, Privacy, UserspaceHID, le marqueur et la configuration TemporaryTimeout.
2. Réveil Home après veille courte/longue : relever Connected, ServicesResolved puis apparition
   du nœud correspondant. Mesurer le délai ; Home sur ce clone reste à confirmer.
3. Si nécessaire, un connect ciblé, manette éveillée puis endormie. Il doit réussir ou échouer
   dans son budget, sans toucher aux associations voisines.
4. Avec Sync et une découverte sans appairage : relever noms, Class, UUID et changements de
   présence ED/E9. Après arrêt du scan, relever InterfacesRemoved et son délai. Pas d’oubli.
5. Plus tard : passage USB Mac/Bluetooth, démarrage radio tardif, deux associations et Xbox de
   référence. La cible proposée est 10 reprises sans réassociation ; le délai HID sera mesuré
   avant de devenir une promesse produit.

Capturer journaux ciblés et btmon avec horodatages, jamais les clés ni le contenu intégral de
`/var/lib/bluetooth`. Les commandes bluetoothctl manuelles doivent correspondre à sa version
installée : son `pair` peut composer plusieurs opérations et remplacer un bond selon version,
à la différence de Device1.Pair. Référence : [API BlueZ](https://bluez.readthedocs.io/en/latest/device-api/).

## Différé et installation

Restriction de same_pad, sélection face faible/association classique existante, retries après
disparition, déplacement du garde btd, reconnexion automatique permanente et fraîcheur des
rapports sont différés. Leurs hypothèses doivent être départagées par les mesures physiques.
Les ordres classique Pair→Connect→Trust et Xbox LE Connect-first restent inchangés.

API_VERSION passe de 44 à 45. Les nouveaux appels et le statut nécessitent une release des
daemons contenant v45 et le robotctl correspondant. Aucune release, installation ou
modification de configuration du robot n’a été faite dans cette session. Les travaux locaux
préexistants sur l’exploration restent distincts.

## Bilan de validation locale

- `cargo test --workspace --exclude nfc` : **1 744 tests réussis**, 28 ignorés.
  Après les dernières corrections JSON et ajouts de cas de timeout/disparition,
  la suite ciblée configd/robotctl/duck-ipc-proto/btd/pad-map/padd/netpadd a été relancée :
  **478 réussis**, 5 ignorés, aucun échec.
- `cargo test -p nfc --lib` : **22 réussis**. Seules ses fixtures Pad ont changé.
- `RUSTFLAGS="-D warnings" cargo +1.94.1 clippy -p configd --all-targets --target aarch64-unknown-linux-gnu` :
  **réussi**, y compris le backend BlueZ et la compilation de ses tests. Les tests Linux n’ont
  pas été exécutés sur une carte ; cette commande est une vérification croisée.
- `cargo +1.94.1 check --workspace --exclude nfc` et `cargo +1.94.1 check -p nfc --lib` :
  **réussis**, avec les avertissements macOS préexistants du pilote IMU de robotd.
- Test CLI réel contre configd `--fake-net --fake-pads`, socket et runtime temporaires :
  status texte/JSON, refus avant association, connexion par MAC, répétition idempotente,
  refus d’une autre adresse et code de sortie non nul du JSON d’échec : **réussis**.
  L’association de ce test est uniquement celle du backend mémoire, sans appel radio.
- Formatage des sources modifiées et `git diff --check` : **réussis**.

Limites préexistantes constatées, laissées intactes :

- `cargo test --workspace` et `cargo +1.94.1 check --workspace` sur macOS échouent sur
  `nfc/src/main.rs:36` : import Spidev non conditionné alors que le type est Linux-only.
  Les variantes excluant ce binaire et le test séparé de sa bibliothèque passent.
- Le Clippy hôte étendu à robotctl/btd/duck-ipc-proto avec `-D warnings` s’arrête sur
  `robotctl/src/monitor.rs:2556`, lint `nonminimal_bool` préexistant. Ce moniteur n’a pas été
  modifié ; le Clippy Linux exigé pour configd passe sans suppression de lint.

Les essais physiques, la vérification de `uniq` sur ce clone et la mesure ED/E9 restent à faire
ensemble. Les résultats simulés ne constituent pas une validation Bluetooth sur graphite-boom.

## Préparation CI

Pour passer le contrôle Clippy strict de la CI, l’expression de fraîcheur du ToF dans
robotctl/src/monitor.rs a été simplifiée conformément au lint nonminimal_bool signalé
précédemment : même traitement de None et même comparaison de durée. Aucun changement
de seuil ni de comportement du moniteur. Les tests du moniteur sont relancés avant le push.


## Version combinée Bluetooth et diagnostics

Le build Bluetooth seul `0.17.9-dev.1451.f169c3d` a été publié et signé par la CI.
La branche intègre désormais les diagnostics jusqu’au commit `38d0331`, déjà validés
sur le robot par la session diagnostics. L’API combinée est 46 : les deux branches
indépendantes avaient chacune utilisé 45. Installer tous les daemons et le robotctl
correspondant préserve les deux ensembles de routes. Aucun déploiement combiné n’a
encore été effectué. La reconnexion explicite depuis une manette déconnectée mais
éveillée reste un essai matériel à terminer.

Les tags de développement contiennent un slash. Avec l’updater actuellement installé,
utiliser les quatre assets signés téléchargés de la release, puis le chemin `--from` ;
le résolveur distant tronque actuellement le tag. La vérification normale des signatures
et les gates de l’updater restent actives.
