importar "documentos";
// Requiere python-docx, reportlab y pypdf instalados para /usr/bin/python3.
intentar {
    documentos.crear("informe.docx", "Informe ST\nDocumento de ejemplo", falso);
    mostrar(documentos.leer("informe.docx"));
    documentos.crear("informe.pdf", "Informe ST\nPDF de ejemplo", falso);
    mostrar(documentos.leer("informe.pdf"));
} capturar error { mostrar(error); }
