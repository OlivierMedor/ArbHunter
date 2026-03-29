import json
import os

W, L = 4, 250000
FEE_BASE = 0.00001
GAS_PRICE = 5e-9
GAS_BASE = 185000
GAS_ADD = 60000

def get_dir(leg):
    e = leg.get('edge', {})
    return str(e.get('token0', '')) + '->' + str(e.get('token1', ''))

def analyze(path):
    if not os.path.exists(path):
        print('Error: not found')
        return
    size = os.path.getsize(path)
    offsets = [int(size * (i / W)) for i in range(W)]
    total_lines = 0
    clusters = []
    for i, offset in enumerate(offsets):
        with open(path, 'r') as f:
            f.seek(offset)
            if offset != 0: f.readline()
            count = 0
            block_map = {}
            while count < L:
                line = f.readline()
                if not line: break
                try:
                    data = json.loads(line)
                    block = data.get('block_number')
                    if block:
                        if block not in block_map: block_map[block] = []
                        block_map[block].append(data)
                except: pass
                count += 1
                total_lines += 1
            for b, ops in block_map.items():
                if len(ops) > 1: clusters.append(ops)
    report = {'summary': {'total_lines_sampled': total_lines, 'groups_considered': len(clusters), 'groups_rejected_pool_overlap': 0, 'same_direction_overlap': 0, 'opposite_direction_overlap': 0, 'package_size_distribution': {}, 'total_uplift_eth': 0.0, 'avg_uplift_per_package_eth': 0.0}, 'top_packages': []}
    total_uplift = 0
    for cluster in clusters:
        s = len(cluster)
        report['summary']['package_size_distribution'][s] = report['summary']['package_size_distribution'].get(s, 0) + 1
        pools = {}
        has_overlap = False
        for op in cluster:
            legs = op.get('path', {}).get('legs', [])
            for leg in legs:
                pid = leg.get('edge', {}).get('pool_id')
                if not pid: continue
                direction = get_dir(leg)
                if pid not in pools: pools[pid] = []
                pools[pid].append(direction)
        overlaps = [p for p, d in pools.items() if len(d) > 1]
        if overlaps:
            has_overlap = True
            report['summary']['groups_rejected_pool_overlap'] += 1
            for p in overlaps:
                dirs = pools[p]
                if all(d == dirs[0] for d in dirs): report['summary']['same_direction_overlap'] += 1
                else: report['summary']['opposite_direction_overlap'] += 1
        if not has_overlap:
            standalone_cost = s * (GAS_BASE * GAS_PRICE + FEE_BASE)
            package_cost = (GAS_BASE + (s - 1) * GAS_ADD) * GAS_PRICE + FEE_BASE
            uplift = standalone_cost - package_cost
            total_uplift += uplift
            report['top_packages'].append({'block': cluster[0]['block_number'], 'size': s, 'uplift': uplift})
    if report['top_packages']:
        report['summary']['avg_uplift_per_package_eth'] = total_uplift / len(report['top_packages'])
        report['summary']['total_uplift_eth'] = total_uplift
    report['top_packages'].sort(key=lambda x: x['uplift'], reverse=True)
    report['top_packages'] = report['top_packages'][:10]
    with open('package_batchability_report.json', 'w') as f:
        json.dump(report, f, indent=2)
    print('Report Generated')
if __name__ == '__main__':
    analyze('historical_replay_full_day_candidates.jsonl')
