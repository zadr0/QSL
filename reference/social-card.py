from pathlib import Path
from PIL import Image,ImageDraw,ImageFont
root=Path(__file__).resolve().parents[1];image=Image.new('RGB',(1200,630),'#f5f6f8');draw=ImageDraw.Draw(image)
font=lambda n:ImageFont.truetype('C:/Windows/Fonts/times.ttf',n)
draw.rectangle((35,35,1164,594),fill='#ffffff',outline='#c9d3df',width=2);draw.rectangle((35,35,1164,42),fill='#234e78')
draw.text((80,65),'QSL',font=font(100),fill='#19232f');draw.text((83,185),'Explicit schemas. Generated Rust.',font=font(49),fill='#19232f');draw.text((85,255),'Binary serialization / UTF-8 / Checked and unchecked APIs',font=font(26),fill='#536171')
for i,label in enumerate(['qsl-core','qsl-lexer','qsl-interpreter','qsl']):
 x=84+i*267;draw.rectangle((x,350,x+231,435),fill='#eef2f6',outline='#aabccc',width=2);draw.text((x+15,372),label,font=font(30),fill='#234e78')
 if i<3:draw.text((x+239,370),'←',font=font(29),fill='#536171')
draw.text((85,486),'RU / EN documentation · Interactive layouts and Criterion data',font=font(28),fill='#536171');draw.text((85,541),'github.com/zadr0/QSL',font=font(25),fill='#234e78')
image.save(root/'docs/assets/social-card-core.png',optimize=True)
