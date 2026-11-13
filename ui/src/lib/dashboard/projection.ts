export type Patch = { set?: any; append?: any[]; items?:Record<string,Patch>; key?: string; changes?: {id: number|string; patch: Patch}[]; fields?: Record<string,Patch>; removed?: (number|string)[] };
/** Immutable application preserves unchanged record identities and array order. */
export function applyPatch(value: any, patch: Patch): any {
	if (Object.hasOwn(patch,'set')) return patch.set;
	if (patch.append) { if(!Array.isArray(value))throw new Error('Invalid append base'); return [...value,...patch.append]; }
	if(patch.items){if(!Array.isArray(value))throw new Error('Invalid array base');const result=value.slice();for(const [key,change] of Object.entries(patch.items)){const i=Number(key);if(!Number.isInteger(i)||i<0||i>=result.length)throw new Error('Invalid array index');result[i]=applyPatch(value[i],change);}return result;}
	if (patch.key) {
		if(!Array.isArray(value))throw new Error('Invalid record base');
		const key=patch.key, removed=new Set(patch.removed), changes=new Map(patch.changes?.map(change=>[change.id,change.patch]));
		const result=[];
		for(const record of value){const id=record[key];if(removed.has(id))continue;const change=changes.get(id);result.push(change?applyPatch(record,change):record);changes.delete(id);}
		for(const change of changes.values())result.push(applyPatch(null,change));
		return result;
	}
	if(!value || typeof value!=='object' || !patch.fields)throw new Error('Invalid object patch');
	const result={...value};for(const key of patch.removed??[])delete result[key];
	for(const [key,change] of Object.entries(patch.fields))result[key]=applyPatch(value[key],change);
	return result;
}
