FROM node:22-alpine
LABEL "language"="nodejs"
LABEL "framework"="next.js"

WORKDIR /app

COPY front/package*.json ./
RUN npm install

COPY front .
RUN npm run build

EXPOSE 3000

CMD ["npm", "start"]
