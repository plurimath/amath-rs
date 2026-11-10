"""Check the independent producer profiles before running parser assertions."""
from pathlib import Path
import hashlib,json
root=Path(__file__).resolve().parents[2]
meta=json.loads((root/'tests/fixtures/raw-profiles-provenance-v1.json').read_text())
def digest(path):return hashlib.sha256(path.read_bytes()).hexdigest()
for key in ['current_payload','generator','archived_fixture']:
    pin=meta[key]
    assert digest(root/pin['path'])==pin['sha256'],key+' checksum changed'
current=json.loads((root/meta['current_payload']['path']).read_text())
archived=json.loads((root/meta['archived_fixture']['path']).read_text())
assert len(current['cases'])==len(archived['cases'])==160
assert current['excluded']==archived['excluded'] and len(current['excluded'])==13
assert current['producer']['engines']==['native','ruby']
assert current['producer']['engine_agreement'] is True
assert current['producer']['commit']=='a4ed231057aeb98022f5c825807e888946a9e71c'
assert current['producer']['source_files']['artifacts/asciimath.json']==digest(root/'artifacts/asciimath.json')
order_only=0
for old,row in zip(archived['cases'],current['cases']):
    assert (old['id'],old['input'],old.get('decimal','.'))==(row['id'],row['input'],row['decimal'])
    assert old['tree']==row['tree'],old['id']+' changed values'
    if json.dumps(old['tree'],ensure_ascii=False)!=json.dumps(row['tree'],ensure_ascii=False):order_only+=1
assert order_only==77
report=json.loads((root/'tests/fixtures/archived-current-member-order-v1.json').read_text())
assert len(report['cases'])==report['count']==order_only
print(json.dumps({'cases':160,'archived_excluded':13,'same_values':160,'order_only':77,'sealed_profiles':True}))
