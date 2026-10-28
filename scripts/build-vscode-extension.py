#!/usr/bin/env python3
"""Genera vocabulario y un VSIX local solo a partir de archivos públicos explícitos."""
import json
import re
import zipfile
from pathlib import Path
from xml.sax.saxutils import escape
ROOT = Path(__file__).resolve().parents[1]
EXT = ROOT / 'extensions/vscode-st'

def build():
    """Sin carpetas temporales compartidas ni borrados: escribe exclusivamente el paquete elegido."""
    lexer = (ROOT/'src/st/lexer.rs').read_text()
    keywords = re.findall(r'\("(\w+)",\s*Keyword::', lexer)
    contextual = re.findall(r'\("(\w+)",\s*"(?:keyword|type)"\)', lexer)
    keywords += contextual
    modules = {}
    for alias, filename in re.findall(r'"([\w.]+)"\s*=>\s*include_str!\("\.\./stdlib/(\w+)\.st"\)', (ROOT/'src/st/stdlib.rs').read_text()):
        modules[alias] = re.findall(r'^funcion\s+(\w+)\s*\(', (ROOT/'src/stdlib'/f'{filename}.st').read_text(), re.M)
    (EXT/'vocabulary.json').write_text(json.dumps({'keywords': keywords, 'modules': modules}, ensure_ascii=False, indent=2)+'\n')
    # TextMate procesa comentarios y cadenas primero; evita tokens semánticos superpuestos.
    grammar={'scopeName':'source.st','patterns':[
        {'begin':'/\\*','end':'\\*/','name':'comment.block.st'},
        {'begin':'"""','end':'"""','name':'comment.block.st'},
        {'match':'(?://|#).*$','name':'comment.line.st'},
        {'begin':'"','end':'"','name':'string.quoted.double.st','patterns':[{'match':r'\\.','name':'constant.character.escape.st'}]},
        {'begin':"'",'end':"'",'name':'string.quoted.single.st','patterns':[{'match':r'\\.','name':'constant.character.escape.st'}]},
        {'match':r'\b(?:'+ '|'.join(keywords)+r')\b','name':'keyword.control.st'},
        {'match':r'\b\d+(?:\.\d+)?\b','name':'constant.numeric.st'},
        {'match':r'\b\w+(?=\s*\()','name':'entity.name.function.st'},
        {'match':r'\b[A-Za-z_]\w*\b','name':'variable.other.st'}]}
    (EXT/'syntaxes/st.tmLanguage.json').write_text(json.dumps(grammar,ensure_ascii=False,indent=2)+'\n')
    pkg=json.loads((EXT/'package.json').read_text())
    identity=escape(pkg['publisher']+'.'+pkg['name']); version=escape(pkg['version'])
    manifest=f'''<?xml version="1.0" encoding="utf-8"?>
<PackageManifest Version="2.0.0" xmlns="http://schemas.microsoft.com/developer/vsx-schema/2011">
<Metadata><Identity Id="{identity}" Version="{version}" Language="es-ES" Publisher="{escape(pkg['publisher'])}"/><DisplayName>PyForge ST</DisplayName><Description xml:space="preserve">Colores, sugerencias y ejecución ST</Description><Properties><Property Id="Microsoft.VisualStudio.Code.Engine" Value="^1.89.0"/></Properties></Metadata>
<Installation><InstallationTarget Id="Microsoft.VisualStudio.Code"/></Installation><Dependencies/>
<Assets><Asset Type="Microsoft.VisualStudio.Code.Manifest" Path="extension/package.json" Addressable="true"/></Assets></PackageManifest>'''
    target=ROOT/'dist/extensions'/f'pyforge-st-{version}.vsix';target.parent.mkdir(parents=True,exist_ok=True)
    with zipfile.ZipFile(target,'w',zipfile.ZIP_DEFLATED) as archive:
        archive.writestr('extension.vsixmanifest',manifest)
        archive.writestr('[Content_Types].xml','<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="json" ContentType="application/json"/><Default Extension="js" ContentType="application/javascript"/><Default Extension="svg" ContentType="image/svg+xml"/><Default Extension="md" ContentType="text/markdown"/><Default Extension="vsixmanifest" ContentType="text/xml"/></Types>')
        for p in sorted(EXT.rglob('*')):
            if p.is_file() and not p.is_symlink() and p.suffix in {'.json','.js','.md','.svg'}:
                archive.write(p,'extension/'+str(p.relative_to(EXT)))
    print(target)

if __name__=='__main__':
    build()
