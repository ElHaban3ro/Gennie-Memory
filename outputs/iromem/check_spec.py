"""Comprobaciones estáticas y SQL del contrato propuesto; no ejecuta iROMEM."""
from pathlib import Path
import hashlib
import json
import re
import sqlite3
import struct
import uuid

BASE = Path(__file__).resolve().parent
NOTE = BASE.parent.parent / 'iROMEM.md'
checks = []

def check(name, condition):
    if not condition:
        raise AssertionError(name)
    checks.append(name)

note = NOTE.read_text()
examples = [json.loads(x) for x in re.findall(r'```json\n(.*?)\n```', note, re.S)]
check('Los seis ejemplos son JSON válido', len(examples) == 6)
memory_required = {'original_text', 'summary', 'type', 'source'}
memory_allowed = memory_required | {'assertion', 'entities', 'conditions', 'occurred_on',
    'valid_from', 'valid_until', 'review_after', 'importance'}
for example in examples:
    if 'action' not in example:
        continue
    action = example['action']
    required = {'schema_version', 'operation_id', 'action', 'memory'}
    required |= {'scope'} if action == 'create' else {'memory_id', 'expected_revision'}
    allowed = required | ({'access', 'topic_key'} if action == 'create' else set())
    check(f'{action}: campos obligatorios y sin campos ajenos',
          required <= example.keys() <= allowed)
    check(f'{action}: UUID válido', str(uuid.UUID(example['operation_id'])) == example['operation_id'])
    memory = example['memory']
    check(f'{action}: contenido según contrato', memory_required <= memory.keys() <= memory_allowed)
    check(f'{action}: fuente identificada', {'kind', 'ref'} <= memory['source'].keys())
    check(f'{action}: longitudes válidas', 1 <= len(memory['original_text'].strip()) <= 16000
          and 1 <= len(memory['summary'].strip()) <= 1600)
    check(f'{action}: tipo y afirmación válidos', memory['type'] in
          {'preference','fact','decision','event','procedure'} and memory['assertion'] in {'direct','inferred'})

db = sqlite3.connect(':memory:')
db.executescript((BASE / 'schema-v1.sql').read_text())
check('Claves foráneas activas', db.execute('PRAGMA foreign_keys').fetchone()[0] == 1)
stamp = '2026-10-15T15:00:00.000Z'
db.execute("INSERT INTO principals VALUES ('user_fer','user')")
db.execute("INSERT INTO scopes VALUES ('user','user_fer')")
db.execute("INSERT INTO scope_members VALUES ('user','user_fer','user_fer',1,1)")
db.execute("INSERT INTO entities VALUES ('person_fer','person','Fernando','user','user_fer')")
db.commit()

def insert_revision(memory_id, revision, memory):
    db.execute('''INSERT INTO memory_revisions
        (memory_id,revision,original_text,summary,type,assertion,source_json,conditions,
         occurred_on,valid_from,valid_until,review_after,importance,created_at)
        VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?)''',
        (memory_id, revision, memory['original_text'], memory['summary'], memory['type'],
         memory['assertion'], json.dumps(memory['source']), memory.get('conditions'),
         memory.get('occurred_on'), memory.get('valid_from'), memory.get('valid_until'),
         memory.get('review_after'), memory['importance'], stamp))
    db.execute('INSERT INTO search_documents(memory_id,revision,original_text,summary) VALUES(?,?,?,?)',
        (memory_id,revision,memory['original_text'],memory['summary']))

def rejected(name, sql, params=()):
    db.execute('SAVEPOINT rejected_case')
    try:
        db.execute(sql, params)
    except sqlite3.IntegrityError:
        db.execute('ROLLBACK TO rejected_case')
        db.execute('RELEASE rejected_case')
        check(name, True)
    else:
        db.execute('ROLLBACK TO rejected_case')
        db.execute('RELEASE rejected_case')
        check(name, False)

create = next(x for x in examples if x.get('action') == 'create')
revise = next(x for x in examples if x.get('action') == 'revise')
with db:
    db.execute('''INSERT INTO memories VALUES (?,?,?,?,?,?,?,?,?,?,?,?)''',
        ('mem_01','user_fer','user','user_fer',create['topic_key'],1,'hash_1','private',0,0,stamp,stamp))
    insert_revision('mem_01',1,create['memory'])
    db.execute("INSERT INTO revision_entities VALUES ('mem_01',1,'person_fer')")
check('Memoria y revisión confirmadas con referencia diferida', db.execute('PRAGMA foreign_key_check').fetchall() == [])
check('FTS5 encuentra una frase original', db.execute(
    '''SELECT count(*) FROM memory_fts WHERE memory_fts MATCH ?''', ('"llames Fer"',)).fetchone()[0] == 1)

rejected('La clave de tema no puede duplicarse en el mismo ámbito',
    '''INSERT INTO memories SELECT 'mem_duplicate',owner_id,scope_kind,scope_id,topic_key,
    current_revision,'different_hash',visibility,sensitive,withdrawn,created_at,updated_at
    FROM memories WHERE memory_id='mem_01' ''')
rejected('La huella exacta no puede duplicarse en el mismo ámbito',
    '''INSERT INTO memories SELECT 'mem_duplicate',owner_id,scope_kind,scope_id,NULL,
    current_revision,current_fingerprint,visibility,sensitive,withdrawn,created_at,updated_at
    FROM memories WHERE memory_id='mem_01' ''')

with db:
    insert_revision('mem_01',2,revise['memory'])
    changed = db.execute('''UPDATE memories SET current_revision=2,current_fingerprint='hash_2'
        WHERE memory_id='mem_01' AND current_revision=1''').rowcount
check('La revisión esperada permite una actualización válida', changed == 1)
check('La revisión antigua no sobrescribe la actual', db.execute('''UPDATE memories
    SET current_fingerprint='wrong' WHERE memory_id='mem_01' AND current_revision=1''').rowcount == 0)
check('La revisión nueva no aparece como coincidencia del apodo antiguo', db.execute('''
    SELECT count(*) FROM memory_fts
    JOIN search_documents d ON d.doc_id=memory_fts.rowid
    JOIN memories m ON m.memory_id=d.memory_id
    WHERE memory_fts MATCH ? AND d.revision=m.current_revision
    ''', ('"llames Fer"',)).fetchone()[0] == 0)
check('El historial conserva la frase original anterior', db.execute('''SELECT count(*)
    FROM memory_fts WHERE memory_fts MATCH ?''', ('"llames Fer"',)).fetchone()[0] == 1)

db.execute('''INSERT INTO embedding_spaces VALUES
    ('space_a','model','revision','artifact','tokenizer','runtime','v1',384,512,NULL)''')
db.execute('''INSERT INTO embedding_jobs VALUES ('mem_01',2,'space_a','pending',0,NULL,NULL,NULL,NULL)''')
check('Guardar no depende de haber calculado los fragmentos semánticos', db.execute(
    "SELECT expected_chunks FROM embedding_jobs WHERE memory_id='mem_01'").fetchone()[0] is None)
db.execute("UPDATE embedding_jobs SET expected_chunks=2 WHERE memory_id='mem_01'")
vector = struct.pack('<384f', 1.0, *([0.0] * 383))
db.execute('''INSERT INTO memory_embeddings VALUES (?,?,?,?,?,?,?,?,?,?)''',
    ('mem_01',2,'space_a','summary',0,0,10,'input_hash',384,vector))
rejected('Rechaza un BLOB con longitud incorrecta',
    'INSERT INTO memory_embeddings VALUES (?,?,?,?,?,?,?,?,?,?)',
    ('mem_01',2,'space_a','original',0,0,10,'input_hash',384,b'bad'))
rejected('Rechaza una dimensión ajena al perfil',
    'INSERT INTO memory_embeddings VALUES (?,?,?,?,?,?,?,?,?,?)',
    ('mem_01',2,'space_a','original',0,0,10,'input_hash',2,struct.pack('<2f',1.0,0.0)))
db.execute('''INSERT INTO operations VALUES (?,?,?,?,?,?,?)''',
    ('user_fer',create['operation_id'],'request_hash','committed','{}',stamp,stamp))
rejected('La operación no puede registrarse dos veces',
    'INSERT INTO operations VALUES (?,?,?,?,?,?,?)',
    ('user_fer',create['operation_id'],'request_hash','committed','{}',stamp,stamp))
db.commit()
with db:
    db.execute("DELETE FROM memories WHERE memory_id='mem_01'")
check('El borrado retira revisiones y vectores por sus relaciones',
    db.execute('SELECT count(*) FROM memory_revisions').fetchone()[0] == 0 and
    db.execute('SELECT count(*) FROM memory_embeddings').fetchone()[0] == 0)
check('El borrado mantiene FTS5 sin entradas fantasma', db.execute(
    "SELECT count(*) FROM memory_fts WHERE memory_fts MATCH 'Fernando'").fetchone()[0] == 0)
check('Las claves foráneas quedan coherentes', db.execute('PRAGMA foreign_key_check').fetchall() == [])
report = {
    'sqlite_version': sqlite3.sqlite_version,
    'scope': 'JSON de ejemplos, esquema SQL y escenarios locales simulados. No ejecuta Rust ni modelos.',
    'passed': checks,
    'count': len(checks),
    'schema_sha256': hashlib.sha256((BASE/'schema-v1.sql').read_bytes()).hexdigest(),
    'note_sha256': hashlib.sha256(NOTE.read_bytes()).hexdigest()
}
(BASE/'validation.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n')
print(json.dumps({'passed':len(checks),'sqlite':sqlite3.sqlite_version,'scope':report['scope']},ensure_ascii=False))
