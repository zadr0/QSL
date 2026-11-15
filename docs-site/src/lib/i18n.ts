import messages from '../data/ui.json';
export function tr(locale:string,key:string):string {return (messages as Record<string,Record<string,string>>)[locale]?.[key]??key;}
