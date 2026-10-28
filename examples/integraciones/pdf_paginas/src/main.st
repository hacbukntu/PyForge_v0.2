importar "documentos";
// Requiere reportlab/pypdf; no edita arbitrariamente texto del PDF existente.
documentos.crear("uno.pdf", "Primera página", falso);
documentos.crear("dos.pdf", "Segunda página", falso);
documentos.unir_pdf("[\"uno.pdf\",\"dos.pdf\"]", "unido.pdf");
documentos.extraer_pdf("unido.pdf", "pagina.pdf", 1);
