INSERT INTO locales (code, display_name, is_enabled)
VALUES
  ('en', 'English', TRUE),
  ('fr', 'Francais', TRUE),
  ('de', 'Deutsch', FALSE),
  ('es', 'Espanol', FALSE)
ON CONFLICT (code) DO NOTHING;

INSERT INTO maintenances (id, code, category_code, start_utc, end_utc, notified_at_utc)
VALUES
  ('7f57fd59-1a95-4517-8f81-c9f755415fce', 'HEA-001', 'HEA', '2020-05-01T07:00:00Z', '2099-05-31T17:00:00Z', '2020-04-29T10:00:00Z'),
  ('e4ddf7f0-ba8d-4eb8-8d15-e25de8f15322', 'ELV-002', 'ELV', '2099-06-12T08:30:00Z', NULL, '2099-05-28T09:00:00Z'),
  ('7f4c63b8-1309-4d50-a420-ac90e8d13f58', 'PMG-003', 'PMG', '2099-06-18T13:30:00Z', NULL, '2099-06-10T11:30:00Z'),
  ('6e2283e0-f95e-4712-b2dd-5f3f6d9fa68f', 'ELC-004', 'ELC', '2020-02-15T08:00:00Z', '2020-02-15T10:00:00Z', NULL),
  ('2a17c890-2cdb-4bbf-81cc-fcf0339f4186', 'PLB-005', 'PLB', '2020-03-21T06:30:00Z', '2020-03-21T12:00:00Z', NULL),
  ('868bfc8f-6d20-495a-bfd2-fa601ceb1df7', 'GAR-006', 'GAR', '2020-01-28T09:00:00Z', '2020-01-28T15:30:00Z', NULL)
ON CONFLICT (id) DO NOTHING;

INSERT INTO incidents (id, code, category_code, start_utc, end_utc)
VALUES
  ('5290b8ea-c194-4200-a4ef-7560ff14d8dc', 'INC-001', 'HEA', '2020-05-01T07:00:00Z', '2099-05-31T17:00:00Z'),
  ('93c4b5fb-e532-4490-91b9-9f211f0a9dc0', 'INC-002', 'ELV', '2020-03-10T09:15:00Z', '2020-03-10T10:05:00Z'),
  ('f3ee5df5-5773-4d35-a0ee-37bb648ec15e', 'INC-003', 'ELC', '2020-02-15T06:40:00Z', '2020-02-15T07:20:00Z'),
  ('f8c74475-0a18-477c-8ffc-ecebb3802a5f', 'INC-004', 'PLB', '2020-01-20T05:30:00Z', '2020-01-20T09:10:00Z')
ON CONFLICT (id) DO NOTHING;

INSERT INTO incident_timeline (id, incident_id, at_utc, sort_order)
VALUES
  ('d5c7668f-2f15-4dad-81ab-6f2dbef7a0fb', '5290b8ea-c194-4200-a4ef-7560ff14d8dc', '2020-05-01T07:05:00Z', 1),
  ('417f73fe-10e7-4869-b359-c27851e11388', '5290b8ea-c194-4200-a4ef-7560ff14d8dc', '2020-05-01T08:10:00Z', 2),
  ('f9cf7c8c-8cb7-4077-b018-f224fd6f500f', '93c4b5fb-e532-4490-91b9-9f211f0a9dc0', '2020-03-10T09:15:00Z', 1),
  ('eeecb2b5-68c8-47f4-a28c-c6f0741adf2a', '93c4b5fb-e532-4490-91b9-9f211f0a9dc0', '2020-03-10T09:48:00Z', 2),
  ('655cf948-8a22-4ea1-9fff-4e6f84511ff9', 'f3ee5df5-5773-4d35-a0ee-37bb648ec15e', '2020-02-15T06:40:00Z', 1),
  ('7510d4ba-0af4-46fc-9cbc-4aa23a3d0f9c', 'f3ee5df5-5773-4d35-a0ee-37bb648ec15e', '2020-02-15T07:20:00Z', 2),
  ('084626fc-c3cd-4e2e-b4fb-10fbe21f9ce7', 'f8c74475-0a18-477c-8ffc-ecebb3802a5f', '2020-01-20T05:30:00Z', 1),
  ('f6e22869-63a7-4f93-a74b-f560bbf6f2c8', 'f8c74475-0a18-477c-8ffc-ecebb3802a5f', '2020-01-20T08:45:00Z', 2)
ON CONFLICT (id) DO NOTHING;

INSERT INTO maintenance_i18n (maintenance_id, locale, field_key, field_value)
VALUES
  ('7f57fd59-1a95-4517-8f81-c9f755415fce', 'en', 'title', 'Heating maintenance in progress'),
  ('7f57fd59-1a95-4517-8f81-c9f755415fce', 'fr', 'title', 'Maintenance chauffage en cours'),
  ('7f57fd59-1a95-4517-8f81-c9f755415fce', 'en', 'short_description', 'Boiler room preventive maintenance'),
  ('7f57fd59-1a95-4517-8f81-c9f755415fce', 'fr', 'short_description', 'Maintenance preventive chaufferie'),
  ('7f57fd59-1a95-4517-8f81-c9f755415fce', 'en', 'warning', 'no hot water between 9h30 & 17h00'),
  ('7f57fd59-1a95-4517-8f81-c9f755415fce', 'fr', 'warning', 'pas d''eau chaude entre 9h30 et 17h00'),
  ('7f57fd59-1a95-4517-8f81-c9f755415fce', 'en', 'long_description', 'Contractor is performing scheduled checks and balancing on the shared heating installation.'),
  ('7f57fd59-1a95-4517-8f81-c9f755415fce', 'fr', 'long_description', 'Le prestataire realise les controles planifies et l''equilibrage de l''installation de chauffage collective.'),
  ('7f57fd59-1a95-4517-8f81-c9f755415fce', 'en', 'location', 'Boiler room - Building B'),
  ('7f57fd59-1a95-4517-8f81-c9f755415fce', 'fr', 'location', 'Chaufferie - Batiment B'),

  ('e4ddf7f0-ba8d-4eb8-8d15-e25de8f15322', 'en', 'title', 'Elevator annual maintenance'),
  ('e4ddf7f0-ba8d-4eb8-8d15-e25de8f15322', 'fr', 'title', 'Maintenance annuelle ascenseur'),
  ('e4ddf7f0-ba8d-4eb8-8d15-e25de8f15322', 'en', 'short_description', 'Planned intervention for safety checks'),
  ('e4ddf7f0-ba8d-4eb8-8d15-e25de8f15322', 'fr', 'short_description', 'Intervention planifiee pour controle securite'),
  ('e4ddf7f0-ba8d-4eb8-8d15-e25de8f15322', 'en', 'long_description', 'The technician will perform annual maintenance and mandatory compliance checks.'),
  ('e4ddf7f0-ba8d-4eb8-8d15-e25de8f15322', 'fr', 'long_description', 'Le technicien effectuera la maintenance annuelle et les controles reglementaires obligatoires.'),
  ('e4ddf7f0-ba8d-4eb8-8d15-e25de8f15322', 'en', 'location', 'Elevator shaft - Building A'),
  ('e4ddf7f0-ba8d-4eb8-8d15-e25de8f15322', 'fr', 'location', 'Cage ascenseur - Batiment A'),

  ('7f4c63b8-1309-4d50-a420-ac90e8d13f58', 'en', 'title', 'Property management site visit'),
  ('7f4c63b8-1309-4d50-a420-ac90e8d13f58', 'fr', 'title', 'Visite du syndic'),
  ('7f4c63b8-1309-4d50-a420-ac90e8d13f58', 'en', 'short_description', 'Follow-up visit with residents committee'),
  ('7f4c63b8-1309-4d50-a420-ac90e8d13f58', 'fr', 'short_description', 'Visite de suivi avec le conseil syndical'),
  ('7f4c63b8-1309-4d50-a420-ac90e8d13f58', 'en', 'long_description', 'Property manager on-site visit to review current actions and upcoming works.'),
  ('7f4c63b8-1309-4d50-a420-ac90e8d13f58', 'fr', 'long_description', 'Visite sur site du syndic pour faire le point sur les actions en cours et les travaux a venir.'),
  ('7f4c63b8-1309-4d50-a420-ac90e8d13f58', 'en', 'location', 'Lobby - Building C'),
  ('7f4c63b8-1309-4d50-a420-ac90e8d13f58', 'fr', 'location', 'Hall - Batiment C'),

  ('6e2283e0-f95e-4712-b2dd-5f3f6d9fa68f', 'en', 'title', 'Electrical panel inspection completed'),
  ('6e2283e0-f95e-4712-b2dd-5f3f6d9fa68f', 'fr', 'title', 'Controle tableau electrique termine'),
  ('6e2283e0-f95e-4712-b2dd-5f3f6d9fa68f', 'en', 'short_description', 'Routine inspection done'),
  ('6e2283e0-f95e-4712-b2dd-5f3f6d9fa68f', 'fr', 'short_description', 'Inspection de routine terminee'),
  ('6e2283e0-f95e-4712-b2dd-5f3f6d9fa68f', 'en', 'long_description', 'Inspection confirmed no critical defects and issued standard recommendations.'),
  ('6e2283e0-f95e-4712-b2dd-5f3f6d9fa68f', 'fr', 'long_description', 'L''inspection confirme l''absence de defaut critique et fournit des recommandations standard.'),
  ('6e2283e0-f95e-4712-b2dd-5f3f6d9fa68f', 'en', 'location', 'Technical room - Building A'),
  ('6e2283e0-f95e-4712-b2dd-5f3f6d9fa68f', 'fr', 'location', 'Local technique - Batiment A'),

  ('2a17c890-2cdb-4bbf-81cc-fcf0339f4186', 'en', 'title', 'Pipe descaling intervention finished'),
  ('2a17c890-2cdb-4bbf-81cc-fcf0339f4186', 'fr', 'title', 'Intervention detartrage canalisations terminee'),
  ('2a17c890-2cdb-4bbf-81cc-fcf0339f4186', 'en', 'short_description', 'Vertical pipe descaling operation'),
  ('2a17c890-2cdb-4bbf-81cc-fcf0339f4186', 'fr', 'short_description', 'Operation de detartrage des colonnes'),
  ('2a17c890-2cdb-4bbf-81cc-fcf0339f4186', 'en', 'long_description', 'Plumbing company completed descaling and pressure tests on shared pipes.'),
  ('2a17c890-2cdb-4bbf-81cc-fcf0339f4186', 'fr', 'long_description', 'L''entreprise de plomberie a termine le detartrage et les tests de pression sur les canalisations communes.'),
  ('2a17c890-2cdb-4bbf-81cc-fcf0339f4186', 'en', 'location', 'Basement technical corridor'),
  ('2a17c890-2cdb-4bbf-81cc-fcf0339f4186', 'fr', 'location', 'Couloir technique sous-sol'),

  ('868bfc8f-6d20-495a-bfd2-fa601ceb1df7', 'en', 'title', 'Garage door motor replacement completed'),
  ('868bfc8f-6d20-495a-bfd2-fa601ceb1df7', 'fr', 'title', 'Remplacement moteur porte de garage termine'),
  ('868bfc8f-6d20-495a-bfd2-fa601ceb1df7', 'en', 'short_description', 'Motor replaced after recurrent faults'),
  ('868bfc8f-6d20-495a-bfd2-fa601ceb1df7', 'fr', 'short_description', 'Moteur remplace apres pannes recurrentes'),
  ('868bfc8f-6d20-495a-bfd2-fa601ceb1df7', 'en', 'long_description', 'The contractor replaced the motor and verified full opening and safety sensors.'),
  ('868bfc8f-6d20-495a-bfd2-fa601ceb1df7', 'fr', 'long_description', 'Le prestataire a remplace le moteur et verifie l''ouverture complete ainsi que les capteurs de securite.'),
  ('868bfc8f-6d20-495a-bfd2-fa601ceb1df7', 'en', 'location', 'Underground garage entrance'),
  ('868bfc8f-6d20-495a-bfd2-fa601ceb1df7', 'fr', 'location', 'Entree garage sous-sol')
ON CONFLICT (maintenance_id, locale, field_key) DO NOTHING;

INSERT INTO incident_i18n (incident_id, locale, field_key, field_value)
VALUES
  ('5290b8ea-c194-4200-a4ef-7560ff14d8dc', 'en', 'title', 'Heating outage on block B'),
  ('5290b8ea-c194-4200-a4ef-7560ff14d8dc', 'fr', 'title', 'Panne chauffage bloc B'),
  ('5290b8ea-c194-4200-a4ef-7560ff14d8dc', 'en', 'short_description', 'Boiler circuit pressure dropped overnight'),
  ('5290b8ea-c194-4200-a4ef-7560ff14d8dc', 'fr', 'short_description', 'Baisse de pression du circuit chaudiere pendant la nuit'),
  ('5290b8ea-c194-4200-a4ef-7560ff14d8dc', 'en', 'long_description', 'A pressure fault triggered an automatic stop of the shared heating loop. Team is restoring circulation and checking valves.'),
  ('5290b8ea-c194-4200-a4ef-7560ff14d8dc', 'fr', 'long_description', 'Une anomalie de pression a provoque l''arret automatique de la boucle de chauffage collective. L''equipe retablit la circulation et controle les vannes.'),
  ('5290b8ea-c194-4200-a4ef-7560ff14d8dc', 'en', 'location', 'Boiler room - Building B'),
  ('5290b8ea-c194-4200-a4ef-7560ff14d8dc', 'fr', 'location', 'Chaufferie - Batiment B'),

  ('93c4b5fb-e532-4490-91b9-9f211f0a9dc0', 'en', 'title', 'Elevator shutdown resolved'),
  ('93c4b5fb-e532-4490-91b9-9f211f0a9dc0', 'fr', 'title', 'Arret ascenseur resolu'),
  ('93c4b5fb-e532-4490-91b9-9f211f0a9dc0', 'en', 'short_description', 'Cabin controller rebooted after fault'),
  ('93c4b5fb-e532-4490-91b9-9f211f0a9dc0', 'fr', 'short_description', 'Redemarrage du controleur cabine apres incident'),
  ('93c4b5fb-e532-4490-91b9-9f211f0a9dc0', 'en', 'long_description', 'The elevator experienced a control board fault and was restarted after safety checks.'),
  ('93c4b5fb-e532-4490-91b9-9f211f0a9dc0', 'fr', 'long_description', 'L''ascenseur a subi un defaut de carte de controle et a ete relance apres controles de securite.'),
  ('93c4b5fb-e532-4490-91b9-9f211f0a9dc0', 'en', 'location', 'Elevator shaft - Building A'),
  ('93c4b5fb-e532-4490-91b9-9f211f0a9dc0', 'fr', 'location', 'Cage ascenseur - Batiment A'),

  ('f3ee5df5-5773-4d35-a0ee-37bb648ec15e', 'en', 'title', 'Generator alert cleared'),
  ('f3ee5df5-5773-4d35-a0ee-37bb648ec15e', 'fr', 'title', 'Alerte generateur levee'),
  ('f3ee5df5-5773-4d35-a0ee-37bb648ec15e', 'en', 'short_description', 'Backup generator alarm acknowledged'),
  ('f3ee5df5-5773-4d35-a0ee-37bb648ec15e', 'fr', 'short_description', 'Alarme groupe electrogene acquittee'),
  ('f3ee5df5-5773-4d35-a0ee-37bb648ec15e', 'en', 'long_description', 'A transient voltage spike triggered a generator warning. Verification found no sustained issue.'),
  ('f3ee5df5-5773-4d35-a0ee-37bb648ec15e', 'fr', 'long_description', 'Une pointe de tension transitoire a declenche une alerte du groupe electrogene. La verification n''a trouve aucun probleme durable.'),
  ('f3ee5df5-5773-4d35-a0ee-37bb648ec15e', 'en', 'location', 'Technical room - Building C'),
  ('f3ee5df5-5773-4d35-a0ee-37bb648ec15e', 'fr', 'location', 'Local technique - Batiment C'),

  ('f8c74475-0a18-477c-8ffc-ecebb3802a5f', 'en', 'title', 'Water leakage in basement closed'),
  ('f8c74475-0a18-477c-8ffc-ecebb3802a5f', 'fr', 'title', 'Fuite d''eau en sous-sol cloturee'),
  ('f8c74475-0a18-477c-8ffc-ecebb3802a5f', 'en', 'short_description', 'Joint replaced on shared pipe'),
  ('f8c74475-0a18-477c-8ffc-ecebb3802a5f', 'fr', 'short_description', 'Joint remplace sur canalisation commune'),
  ('f8c74475-0a18-477c-8ffc-ecebb3802a5f', 'en', 'long_description', 'A leak was isolated, damaged joint replaced, and pressure tests returned to nominal levels.'),
  ('f8c74475-0a18-477c-8ffc-ecebb3802a5f', 'fr', 'long_description', 'Une fuite a ete isolee, le joint endommage remplace, et les tests de pression sont revenus a la normale.'),
  ('f8c74475-0a18-477c-8ffc-ecebb3802a5f', 'en', 'location', 'Basement corridor'),
  ('f8c74475-0a18-477c-8ffc-ecebb3802a5f', 'fr', 'location', 'Couloir sous-sol')
ON CONFLICT (incident_id, locale, field_key) DO NOTHING;

INSERT INTO incident_timeline_i18n (timeline_id, locale, field_key, field_value)
VALUES
  ('d5c7668f-2f15-4dad-81ab-6f2dbef7a0fb', 'en', 'title', 'Issue detected by monitoring system'),
  ('d5c7668f-2f15-4dad-81ab-6f2dbef7a0fb', 'fr', 'title', 'Incident detecte par le systeme de supervision'),
  ('d5c7668f-2f15-4dad-81ab-6f2dbef7a0fb', 'en', 'details', 'Temperature fell below threshold in two risers.'),
  ('d5c7668f-2f15-4dad-81ab-6f2dbef7a0fb', 'fr', 'details', 'La temperature est passee sous le seuil sur deux colonnes.'),
  ('417f73fe-10e7-4869-b359-c27851e11388', 'en', 'title', 'Technician dispatched'),
  ('417f73fe-10e7-4869-b359-c27851e11388', 'fr', 'title', 'Technicien depose'),

  ('f9cf7c8c-8cb7-4077-b018-f224fd6f500f', 'en', 'title', 'Residents reported cabin stuck'),
  ('f9cf7c8c-8cb7-4077-b018-f224fd6f500f', 'fr', 'title', 'Signalement cabine bloquee'),
  ('eeecb2b5-68c8-47f4-a28c-c6f0741adf2a', 'en', 'title', 'System reboot completed'),
  ('eeecb2b5-68c8-47f4-a28c-c6f0741adf2a', 'fr', 'title', 'Redemarrage systeme termine'),

  ('655cf948-8a22-4ea1-9fff-4e6f84511ff9', 'en', 'title', 'Alert raised'),
  ('655cf948-8a22-4ea1-9fff-4e6f84511ff9', 'fr', 'title', 'Alerte declenchee'),
  ('7510d4ba-0af4-46fc-9cbc-4aa23a3d0f9c', 'en', 'title', 'Inspection completed'),
  ('7510d4ba-0af4-46fc-9cbc-4aa23a3d0f9c', 'fr', 'title', 'Inspection terminee'),

  ('084626fc-c3cd-4e2e-b4fb-10fbe21f9ce7', 'en', 'title', 'Leak signal received'),
  ('084626fc-c3cd-4e2e-b4fb-10fbe21f9ce7', 'fr', 'title', 'Signalement fuite recu'),
  ('f6e22869-63a7-4f93-a74b-f560bbf6f2c8', 'en', 'title', 'Repair completed'),
  ('f6e22869-63a7-4f93-a74b-f560bbf6f2c8', 'fr', 'title', 'Reparation terminee')
ON CONFLICT (timeline_id, locale, field_key) DO NOTHING;

INSERT INTO translation_keys (key_name, description)
VALUES
  ('nav.events', 'Navigation label for maintenance/events section'),
  ('nav.incidents', 'Navigation label for incidents section')
ON CONFLICT (key_name) DO NOTHING;

INSERT INTO translation_values (key_name, locale, value)
VALUES
  ('nav.events', 'en', 'Events & Maintenance'),
  ('nav.events', 'fr', 'Evenements et maintenance'),
  ('nav.incidents', 'en', 'Incidents'),
  ('nav.incidents', 'fr', 'Incidents')
ON CONFLICT (key_name, locale) DO NOTHING;
