# Revue indépendante du plan « gamepad-reliability-plan-2026-10-27 »

Révision lue : `autonomy-dev` 96b89d8 (0.17.9). J'ai seulement lu des fichiers : rien n'a été modifié, rien n'a été lancé sur le robot.

**Verdict.** Le diagnostic du plan est solide. En revanche, son contenu ne respecte plus le périmètre décidé (connexion seulement). Le plan propose aussi plus de mécanique que nécessaire pour `pad connect`. Avec quelques corrections, deux tranches suffisent (statut honnête, puis `pad connect <MAC>`), livrées avec un seul bump d'API.

---

## 1. Findings priorisés

### P0 — Les sorties actuelles poussent vers la mauvaise action

1. **« Déjà appairé » ne fait que poser Trusted, puis promet une connexion.**
   - `bond()` saute les deux branches quand `device.paired` (`autonomy-dev/configd/src/bluez.rs:670`, `:713`) et ne fait que `set_trusted` (`:790`).
   - `report_pair` affiche ensuite « padd is driving from it now » ou « it will connect on its own in a moment » (`autonomy-dev/robotctl/src/main.rs:5010-5016`).
   - Conséquence : `pad pair 98:B6:E9:A4:FC:67` après l'incident aurait répondu « paired… will connect » sans avoir appelé Connect.
   - Le même défaut touche nfcd : un tag portant la MAC d'une manette associée mais déconnectée joue le son de réussite (`autonomy-dev/nfc/src/pairing.rs:92-103`).
   - C'est l'argument principal pour `pad connect`.

2. **UnknownObject est classé `Rejected`, et le conseil affiché envoie vers Privacy et un reboot.**
   - Toute erreur de `Pair()` devient `Rejected` (`bluez.rs:690`, `:766`).
   - Le conseil affiché demande de vérifier `Privacy`, de relancer `setup-board.sh` et de redémarrer (`robotctl/src/main.rs:5037-5043`).
   - Dans l'incident, c'est exactement ce que l'utilisateur a lu : un conseil de configuration intrusif, pour un objet D-Bus disparu.
   - Le plan ne relève pas ce point.

3. **Avant `hci0`, ou si BlueZ est inaccessible, le statut conseille de réassocier.**
   - Quand aucun adaptateur n'existe encore (≈73 s au boot), la liste des associations est vide. Le rendu affiche alors « none paired — run: sudo robotctl pad pair » (`robotctl/src/main.rs:4959-4961`).
   - Si le bus système est absent, configd se rabat sur `FakePads::with(Vec::new())` (`autonomy-dev/configd/src/main.rs:224-229`), avec le même message « none paired ».
   - Le commentaire `:222-223` est faux : `BlueZ::new` ouvre seulement le bus (`bluez.rs:496-504`), un `hci0` absent ne déclenche pas ce fallback.

4. **Un succès peut être inventé.**
   - Si la relecture finale ne trouve plus l'objet, le code construit un `Pad { paired: true, trusted: true }` (`bluez.rs:1045-1049`).
   - Le plan le relève (ligne 99). C'est à corriger dans la tranche statut, pas plus tard.

### P1 — Sécurité et cohérence

5. **`pad pair` coupe l'adaptateur, y compris sur une manette déjà associée.**
   - Sur une carte qui porte le marqueur weird-ble, `BtdPaused::for_pairing` arrête btd et fait power off/on (`robotctl/src/main.rs:3609-3638`, `:3656-3669`).
   - Une manette en train de piloter est donc déconnectée, ce qui déclenche PadLoss puis une demande d'assise, c'est-à-dire un mouvement.
   - Le plan écrit « peut perturber la radio » (ligne 95), ce qui sous-estime le risque. C'est une raison forte pour que `connect` ne passe jamais par `pair`.
   - Il faut aussi interdire `pad pair` robot debout pendant les essais.

6. **Le chronométrage de l'incident permet une inférence que le plan ne fait pas.**
   - 07:01:05 + 20 s de découverte = 07:01:25, plus 30 s de `BOND_TIMEOUT` sur Connect = 07:01:55.
   - La découverte a donc probablement couru jusqu'à son terme. Or un candidat « Classified » non appairé l'arrête après 1 s (`bluez.rs:612-623`, `SIBLING_GRACE` `:234`).
   - ED était donc très probablement un match par **nom seul**, sans Icon, Class ni Appearance. C'est le même profil que la face LE de septembre (`bluez.rs:144-154`).
   - Cela renforce l'hypothèse sans prouver l'identité ED/E9. Réserve : ED a pu apparaître dans la dernière seconde.

7. **Hypothèse à vérifier : objet temporaire évincé par BlueZ.**
   - La découverte est arrêtée avant `bond()` (`bluez.rs:888`).
   - Les versions récentes de BlueZ suppriment les objets temporaires (non appairés) après `TemporaryTimeout` (main.conf, 30 s par défaut si ma mémoire est bonne).
   - La coïncidence avec les 30 s de Connect (`BOND_TIMEOUT`, `bluez.rs:212`) mérite une mesure : version installée, valeur de `TemporaryTimeout`, et horodatage de `InterfacesRemoved`.
   - À présenter comme hypothèse. Je ne l'ai pas vérifié dans la source BlueZ.

8. **Le statut masque des faits de connexion utiles.**
   - Les associations qui ne passent pas l'heuristique gamepad sont filtrées (`bluez.rs:845`). Un appareil appairé via `pad pair <MAC>` hors heuristique devient invisible.
   - `Snapshot` ne lit ni `ServicesResolved` ni l'état de l'adaptateur (`bluez.rs:402-412`).
   - `PadStatusResult` ne porte que `pads` et `driver: padd` (`autonomy-dev/duck-ipc-proto/src/lib.rs:4548-4552`), alors que `units::MANAGED` connaît déjà `netpadd.service` (`autonomy-dev/configd/src/units.rs:42-52`).

9. **Le Connect classique après Pair échoue en silence.**
   - Après un Pair classique, l'échec du Connect est seulement journalisé en warn (`bluez.rs:708-712`), et le résultat ne le transmet pas.
   - C'est cohérent avec « après oubli et nouvelle association, un connect explicite a été nécessaire ». Le journal de cette étape manque, donc la cause n'est pas établie.

### P2

10. **L'heuristique `same_pad` est trop large.**
    - Elle compare n'importe quels trois derniers octets (`autonomy-dev/configd/src/pad.rs:181-194`), et `one_face_per_pad` ne voit que les candidats non appairés (`bluez.rs:938-939`).
    - Le constat du plan est exact. Ce n'est pas la cause de l'incident : même si la fusion incluait les appairés, ED serait devenu E9 appairé, donc Trusted seul, sans Connect.
    - Tranche 3 différable.

11. **La documentation se contredit, comme le plan l'annonce.**
    - Privacy : `bluez.rs:75-79` dit « device sur toutes les cartes » ; `docs/robot/pair-a-gamepad.md:132-138` dit « device seulement avec --weird-ble, essayer la pause d'abord » ; `:211` dit « should read device ».
    - Transport : « `transport` est LE sur tous les pads essayés » (`pair-a-gamepad.md:342`), alors que la Pro Controller est BR/EDR (`:20`).
    - Le parcours manuel `bluetoothctl pair` (`pair-a-gamepad.md:186-200`) dépend de la sémantique de la version installée.

---

## 2. Affirmations du plan à corriger

| Ligne(s) | Problème | Correction |
|---|---|---|
| 139-141 | Les fichiers `padd/src/{main,tap}.rs` relèvent du niveau entrée. | Retirer padd/tap de la tranche 1. |
| 144-148 | Pad choisi par gilrs, event node, compteur et âge du rapport, mode du mapper, policy, prise en main : hors périmètre décidé. | Garder adaptateur, association, confiance, Connected, ServicesResolved, HID reconnu, source, padd et netpadd. |
| 152-160 | Snapshot du tap, « entrée validée » par compteur, « + = Start ». | Déplacer dans une section « Différé (niveau entrée/mode) ». |
| 163-165 | « mode tête explique l'absence de marche ». | À supprimer des critères de réussite. |
| 221, 230 | Le parcours « source, radio, entrée et mode », puis « status confirme l'entrée, affiche policy et protection ». | Remplacer par « confirme liaison et HID ». |
| 250-255, 274 | Rendus « mode tête / rapport ancien / tap » ; replays gilrs/mapper ; « entrée < 10 s, premier rapport ». | Différer, ou reformuler en délai jusqu'au nœud HID. |
| 50-51 | Un avertissement « différé » ne suffit pas : le corps contredit le périmètre. | Déplacer physiquement les passages concernés. |
| 170 | `pad connect --mac …` | Utiliser un argument positionnel `pad connect <MAC>`, comme `pair` et `forget` (`robotctl/src/main.rs:987`, `:1003`). |
| 171-173 | Sans adresse, sélection automatique si une seule association. | MAC obligatoire en v1. La commande reste simple et sans choix implicite. |
| 177-178 | Redécouverte bornée en cas d'UnknownObject sur connect. | Inutile : l'objet d'un appareil appairé est persistant sous l'adaptateur. S'il manque, c'est « non associé sur cet adaptateur » ou « adaptateur absent » : refuser sans scanner. |
| 98 | « Paired surveillé en parallèle » | Vrai pour les appels Pair (`bluez.rs:686-694`, `:761-770`), pas pour Connect. Le Connect LE de 30 s n'est pas mis en course. |
| 187 | « Home suffit normalement à réveiller » | C'est une hypothèse à mesurer (essai 2), pas un constat. |
| 42-46 | Sémantique composite de `bluetoothctl pair`, qui pourrait supprimer un bond. | Je ne peux pas la vérifier ici (pas d'accès web). La page citée est celle de Debian *testing*, pas forcément la version de la carte. À présenter comme « à confirmer avec `bluetoothctl --version` et `man bluetoothctl` sur la carte ». La distinction avec le D-Bus `Pair` reste juste : configd appelle `Device1.Pair` seul (`bluez.rs:258`). |
| 106-108 | Device1 : Connect réussit si au moins un profil se connecte ; Connected et ServicesResolved sont distincts. | Conforme à ce que je connais de l'API, mais non revérifié en ligne dans cette session. |
| 23-24, 60 | La preuve est un journal dans une autre session (`codex://…`), introuvable dans le dépôt. | Recopier les quelques lignes brutes horodatées dans le document daté, adresses comprises, sans clés. |

Les affirmations suivantes sont exactes au vu du code : premier match par adresse explicite (`bluez.rs:612-613`), priorité au non-appairé (`:938`), plus petite MAC si tous sont appairés (`:945-949`), Class comme seul discriminant classique/LE (`:447-449`), « Class absent n'établit pas LE », garde btd seulement côté CLI (`robotctl/src/main.rs:4910`), ExecCondition inverses (`padd.service:49`, `netpadd.service:19`).

---

## 3. Les séquences, mises à l'épreuve

**Classique : Pair → Connect → Trust.** L'ordre Pair→Connect est justifié par des mesures (`bluez.rs:132-142`). En revanche, poser Trusted *après* Connect n'a pas de justification. Juste après le bond, une manette classique peut elle-même ouvrir le HID, et l'autorisation repose alors sur l'agent temporaire. Proposition à tester, sans risque : Pair → Trusted → Connect. Trusted est un simple drapeau local, et l'ordre radio reste inchangé.

**LE / Xbox : Connect → attente Paired → Pair si besoin → Trust.** À conserver comme exception mesurée. Deux réserves :
- Le budget de Connect (30 s) est démesuré face au « < 1 s » mesuré sur Xbox (`bluez.rs:216-219`). Sur une face nom seul, il coûte 30 s et peut-être l'objet lui-même (point 7).
- Ne pas changer ce budget sans mesurer la Xbox de référence.

**Pire cas cumulé.** LE : fenêtre + 30 + 5 + 30 + 30 s. Classique : fenêtre + 30 + 30 s. Le plan le note. Il n'y a pas de timeout côté client robotctl, l'appel est bloquant.

**HID reconnu.** Aucune validation aujourd'hui. Le plus simple, dans le périmètre : configd (root, lecture sysfs non masquée par `PrivateDevices=`) cherche un nœud input ou HID dont `uniq` correspond à la MAC, sans tenir compte de la casse.
- À **vérifier sur la carte** : hidp (avec `UserspaceHID=false`) et HoG renseignent bien `uniq` avec l'adresse BD, et hid-nintendo ne la remplace pas sur le clone.
- Si ce n'est pas le cas, afficher « HID : correspondance non établie », jamais une corrélation par suffixe.

---

## 4. Le plus petit plan cohérent

Une seule montée d'`API_VERSION` (44 → 45) et une release, conformément à AGENTS.md.

**A. Sorties honnêtes, sans changement de protocole (P0)**
- `report_pair` :
  - si la manette était déjà associée : « déjà associée ; liaison non établie → `pad connect <MAC>` » ;
  - si elle est associée mais non connectée : afficher l'état réel, sans promesse.
- UnknownObject sur Pair : ne plus le classer `Rejected`. Le mapper sur un message « l'appareil a disparu avant l'association », qui liste les associations existantes vues, sans substitution.
- Supprimer le fallback qui invente un Pad (`bluez.rs:1045`) et le remplacer par « vérification impossible ».

**B. `pad status` limité à la connexion (P0, change le protocole)**
- Champs à ajouter : adaptateur (absent / éteint / présent), par association (paired, trusted, connected, services_resolved, hid oui/non/inconnu), source `netpad.enabled`, état de padd **et** de netpadd, avec un conflit visible si les deux tournent.
- Ajouts en `#[serde(default)]`, erreurs partielles rendues « inconnu ». Si BlueZ est inaccessible, les services restent affichés.
- Fichiers : `configd/src/{bluez,main,units}.rs`, `duck-ipc-proto`, `robotctl/src/main.rs`.

**C. `pad connect <MAC>` (P1)**
- Méthode `pad.connect`, mutante (gate `may_mutate`), de type `(Config, Operation)`. Décision explicite à prendre dans `btd/src/route.rs:201-203` sur l'accès par téléphone.
- Prend le même mutex que pair (`bluez.rs:492`). Pas de découverte, pas d'agent, pas de BtdPaused, pas de power-cycle.
- Déroulé :
  - MAC introuvable ou non appairée : refus.
  - Déjà connectée : rien à faire, réponse idempotente.
  - Sinon : Trusted si absent, puis Connect avec un budget de 15 s. InProgress donne lieu à une observation de Connected jusqu'à l'échéance ; AlreadyConnected compte comme succès ; NotReady signifie adaptateur.
- Réponse : état relu, avec l'erreur BlueZ et la phase.
- Une seule réserve : `adapter()` allume l'adaptateur s'il est éteint (`bluez.rs:547-551`). Il faut l'annoncer ou le refuser.

**Différé :** restriction de `same_pad` et retries (tranche 3), garde btd déplacé vers configd, niveau entrée/mode/fraîcheur, refus d'un candidat nom-seul face à des associations existantes. Ce dernier point est utile mais modifie la sélection : à décider après l'essai 4.

---

## 5. Essais physiques à faire ensemble (sans mouvement, chaque étape sur ton feu vert)

Préparation : robot posé, policy off, protection de prise en main active. `btmon -t` en capture, `journalctl -f -u configd -u bluetooth`, et `dbus-monitor --system` filtré sur ObjectManager. Relever la version de bluetoothctl, `Privacy`, `TemporaryTimeout`, `UserspaceHID` et la présence du marqueur weird-ble. Aucune modification de configuration.

1. **Lecture seule.**
   - `robotctl pad status` et `systemctl is-active padd netpadd`.
   - `bluetoothctl info` sur E9:A4:FC:67, E9:64:FD:06 et ED:A4:FC:67.
   - `grep -H . /sys/class/input/*/uniq`, manette éteinte puis allumée.
2. **Réveil par Home sur E9:A4 (3 cycles, veille courte puis longue).**
   - Qui initie la connexion (page venant de la manette ou de l'hôte) ? Délai jusqu'à Connected, puis ServicesResolved, puis apparition du nœud dont `uniq` correspond à la MAC.
   - Cet essai valide ou non le critère « HID reconnu ».
3. **Si la manette ne revient pas : un seul `bluetoothctl connect 98:B6:E9:A4:FC:67`, manette éveillée puis endormie.**
   - Équivaut au futur `pad connect`. On mesure la durée et l'erreur (page timeout ?).
   - C'est la preuve directe de la nécessité de la commande.
4. **Identité ED/E9 (sans associer).**
   - Manette en Sync, scan de 20 s : ED apparaît-il, sous quel nom et avec quels UUID ? L'objet E9 reçoit-il des mises à jour RSSI, signe de présence réelle ?
   - Puis extinction : les deux disparaissent-elles ensemble ?
5. **Éviction.** Après l'arrêt du scan, mesurer le délai jusqu'au `InterfacesRemoved` d'ED, et le comparer à `TemporaryTimeout`.
6. **Effet du Sync sur le bond existant.** Après un Sync interrompu, Home suffit-il encore à reconnecter E9 sans réassociation ?
7. **Plus tard, séparément :** Xbox de référence, aller-retour USB Mac, et l'association E9:64:FD:06 qui reste intacte.

Interdits pendant ces essais : `pad pair` robot debout, `forget`, redémarrage de bluetooth.
