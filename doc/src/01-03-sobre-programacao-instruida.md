# Sobre Programação Instruída

Para garantir a reproducibilidade do projeto, optei aqui por utilizar o conceito de *literate programming*, como demonstrado por <knuth1984>, que aqui traduzo livremente como *programação instruída*, dado seu objetivo de instruir o leitor, passo-a-passo, no desenvolvimento de uma aplicação.

Este interpretador foi inteiramente escrito utilizando o formato de texto Org, no editor de texto Emacs. De acordo com o website do Org<sup><a id="fnr.1" class="footref" href="#fn.1" role="doc-backlink">1</a></sup>, trata-se de \`\`um formato para realizar anotações, manter listas de tarefas a serem feitas, planejar projetos, e criar documentos com um sistema de texto-plano rápido e efetivo''. A prosa é escrita ao longo do arquivo, e são inseridos blocos de código que foram configurados para serem escritos em seus respectivos e apropriados arquivos posteriormente.

O código possui estrutura e organização que podem não seguir fielmente o conteúdo deste texto. Sendo assim, tal código é exportado posteriormente, através de um processo conhecido como *entrelaçamento* (*tangling*). Ao utilizar este método, espero manter uma aplicação onde o entendimento do que está sendo escrito venha antes do código em si, de forma que o leitor possa timar e analisar partes do código com base na prosa que as acompanha.

## Footnotes

<sup><a id="fn.1" class="footnum" href="#fnr.1">1</a></sup> <https://orgmode.org/>